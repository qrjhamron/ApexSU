// SPDX-License-Identifier: GPL-2.0
#include <linux/module.h>
#include <linux/fs.h>
#include <linux/namei.h>
#include <linux/fsnotify_backend.h>
#include <linux/slab.h>
#include <linux/rculist.h>
#include <linux/version.h>
#include <linux/mutex.h>
#include <linux/spinlock.h>
#include <linux/workqueue.h>
#include "klog.h" // IWYU pragma: keep
#include "throne_tracker.h"

#define MASK_SYSTEM (FS_CREATE | FS_MOVE | FS_EVENT_ON_CHILD)

struct watch_dir {
    const char *path;
    u32 mask;
    struct path kpath;
    struct inode *inode;
    struct fsnotify_mark *mark;
};

static struct fsnotify_group *g;
static bool observer_initialized;
static DEFINE_MUTEX(observer_lock);
static DEFINE_SPINLOCK(pkg_workqueue_lock);
static struct workqueue_struct *pkg_observer_wq;
static bool pkg_observer_wq_stopping = true;

static void pkg_observer_workfn(struct work_struct *work)
{
    track_throne(false);
}

static DECLARE_WORK(pkg_observer_work, pkg_observer_workfn);

int ksu_pkg_observer_workqueue_init(void)
{
    struct workqueue_struct *wq;
    unsigned long flags;

    wq = alloc_ordered_workqueue("ksu_pkg_observer", WQ_MEM_RECLAIM);
    if (!wq)
        return -ENOMEM;

    spin_lock_irqsave(&pkg_workqueue_lock, flags);
    if (pkg_observer_wq) {
        spin_unlock_irqrestore(&pkg_workqueue_lock, flags);
        destroy_workqueue(wq);
        return 0;
    }
    pkg_observer_wq = wq;
    pkg_observer_wq_stopping = false;
    spin_unlock_irqrestore(&pkg_workqueue_lock, flags);

    return 0;
}

void ksu_pkg_observer_workqueue_exit(void)
{
    struct workqueue_struct *wq;
    unsigned long flags;

    spin_lock_irqsave(&pkg_workqueue_lock, flags);
    wq = pkg_observer_wq;
    pkg_observer_wq_stopping = true;
    spin_unlock_irqrestore(&pkg_workqueue_lock, flags);

    cancel_work_sync(&pkg_observer_work);

    if (wq)
        destroy_workqueue(wq);

    spin_lock_irqsave(&pkg_workqueue_lock, flags);
    if (pkg_observer_wq == wq)
        pkg_observer_wq = NULL;
    spin_unlock_irqrestore(&pkg_workqueue_lock, flags);
}

static void queue_pkg_observer_work(void)
{
    struct workqueue_struct *wq;
    unsigned long flags;
    bool queued;

    spin_lock_irqsave(&pkg_workqueue_lock, flags);
    wq = pkg_observer_wq;
    if (!wq || pkg_observer_wq_stopping) {
        spin_unlock_irqrestore(&pkg_workqueue_lock, flags);
        pr_debug("pkg observer workqueue is not available\n");
        return;
    }
    queued = queue_work(wq, &pkg_observer_work);
    spin_unlock_irqrestore(&pkg_workqueue_lock, flags);

    if (!queued)
        pr_debug("pkg observer work already pending\n");
}

static int ksu_handle_inode_event(struct fsnotify_mark *mark, u32 mask,
                                  struct inode *inode, struct inode *dir,
                                  const struct qstr *file_name, u32 cookie)
{
    if (!file_name)
        return 0;
    if (mask & FS_ISDIR)
        return 0;
    if (file_name->len == 13 && !memcmp(file_name->name, "packages.list", 13)) {
        pr_info("packages.list detected: %d\n", mask);
        queue_pkg_observer_work();
    }
    return 0;
}

static const struct fsnotify_ops ksu_ops = {
    .handle_inode_event = ksu_handle_inode_event,
};

static int add_mark_on_inode(struct inode *inode, u32 mask,
                             struct fsnotify_mark **out)
{
    struct fsnotify_mark *m;

    m = kzalloc(sizeof(*m), GFP_KERNEL);
    if (!m)
        return -ENOMEM;

    fsnotify_init_mark(m, g);
    m->mask = mask;

    if (fsnotify_add_inode_mark(m, inode, 0)) {
        fsnotify_put_mark(m);
        return -EINVAL;
    }
    *out = m;
    return 0;
}

static int watch_one_dir(struct watch_dir *wd)
{
    int ret = kern_path(wd->path, LOOKUP_FOLLOW, &wd->kpath);
    if (ret) {
        pr_info("path not ready: %s (%d)\n", wd->path, ret);
        return ret;
    }
    wd->inode = d_inode(wd->kpath.dentry);
    ihold(wd->inode);

    ret = add_mark_on_inode(wd->inode, wd->mask, &wd->mark);
    if (ret) {
        pr_err("Add mark failed for %s (%d)\n", wd->path, ret);
        path_put(&wd->kpath);
        iput(wd->inode);
        wd->inode = NULL;
        return ret;
    }
    pr_info("watching %s\n", wd->path);
    return 0;
}

static void unwatch_one_dir(struct watch_dir *wd)
{
    if (wd->mark && g && !IS_ERR(g)) {
        fsnotify_destroy_mark(wd->mark, g);
        fsnotify_put_mark(wd->mark);
        wd->mark = NULL;
    }
    if (wd->inode) {
        iput(wd->inode);
        wd->inode = NULL;
    }
    if (wd->kpath.dentry) {
        path_put(&wd->kpath);
        memset(&wd->kpath, 0, sizeof(wd->kpath));
    }
}

static struct watch_dir g_watch = { .path = "/data/system",
                                    .mask = MASK_SYSTEM };

static struct fsnotify_group *ksu_fsnotify_alloc_group(void)
{
#ifdef FSNOTIFY_GROUP_USER
    return fsnotify_alloc_group(&ksu_ops, 0);
#else
    return fsnotify_alloc_group(&ksu_ops);
#endif
}

int ksu_observer_init(void)
{
    int ret = 0;

    mutex_lock(&observer_lock);
    if (observer_initialized) {
        mutex_unlock(&observer_lock);
        return 0;
    }

    if (!pkg_observer_wq) {
        mutex_unlock(&observer_lock);
        pr_err("observer init: workqueue is not initialized\n");
        return -ENOMEM;
    }

    g = NULL;
    g = ksu_fsnotify_alloc_group();
    if (IS_ERR(g)) {
        ret = PTR_ERR(g);
        g = NULL;
        pr_err("observer init: fsnotify_alloc_group failed: %d\n", ret);
        mutex_unlock(&observer_lock);
        return ret;
    }

    ret = watch_one_dir(&g_watch);
    if (ret) {
        unwatch_one_dir(&g_watch);
        fsnotify_put_group(g);
        g = NULL;
        pr_err("observer init: watch setup failed: %d\n", ret);
        mutex_unlock(&observer_lock);
        return ret;
    }
    observer_initialized = true;
    pr_info("observer init done\n");
    mutex_unlock(&observer_lock);
    return 0;
}

void ksu_observer_exit(void)
{
    mutex_lock(&observer_lock);
    if (!observer_initialized) {
        if (g && !IS_ERR(g)) {
            fsnotify_put_group(g);
            g = NULL;
        }
        mutex_unlock(&observer_lock);
        return;
    }

    unwatch_one_dir(&g_watch);
    if (g && !IS_ERR(g)) {
        fsnotify_put_group(g);
        g = NULL;
    }
    observer_initialized = false;
    memset(&g_watch.kpath, 0, sizeof(g_watch.kpath));
    g_watch.inode = NULL;
    g_watch.mark = NULL;
    mutex_unlock(&observer_lock);
    pr_info("observer exit done\n");
}
