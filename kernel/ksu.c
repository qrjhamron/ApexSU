// SPDX-License-Identifier: GPL-2.0
#include <linux/export.h>
#include <linux/fs.h>
#include <linux/kobject.h>
#include <linux/module.h>
#include <linux/workqueue.h>
#include <linux/atomic.h>

#include "allowlist.h"
#include "feature.h"
#include "klog.h" // IWYU pragma: keep
#include "throne_tracker.h"
#include "syscall_hook_manager.h"
#include "ksud.h"
#include "supercalls.h"
#include "ksu.h"
#include "file_wrapper.h"

struct cred *ksu_cred;
static atomic_t ksu_module_shutting_down = ATOMIC_INIT(0);
#ifdef MODULE
static struct list_head *ksu_module_prev;
#endif

extern void ksu_observer_exit(void);
extern int ksu_pkg_observer_workqueue_init(void);
extern void ksu_pkg_observer_workqueue_exit(void);

bool ksu_module_is_shutting_down(void)
{
    return atomic_read(&ksu_module_shutting_down) != 0;
}

bool ksu_task_work_prepare_enqueue(void)
{
#ifdef MODULE
    if (unlikely(ksu_module_is_shutting_down()))
        return false;

    if (!try_module_get(THIS_MODULE))
        return false;

    if (unlikely(ksu_module_is_shutting_down())) {
        module_put(THIS_MODULE);
        return false;
    }
#endif
    return true;
}

void ksu_task_work_complete(void)
{
#ifdef MODULE
    module_put(THIS_MODULE);
#endif
}

int __init kernelsu_init(void)
{
    int ret;

    atomic_set(&ksu_module_shutting_down, 0);

#ifdef CONFIG_KSU_DEBUG
    pr_alert("*************************************************************");
    pr_alert("**     NOTICE NOTICE NOTICE NOTICE NOTICE NOTICE NOTICE    **");
    pr_alert("**                                                         **");
    pr_alert("**         You are running in DEBUG mode                    **");
    pr_alert("**                                                         **");
    pr_alert("**     NOTICE NOTICE NOTICE NOTICE NOTICE NOTICE NOTICE    **");
    pr_alert("*************************************************************");
#endif

    ksu_cred = prepare_creds();
    if (!ksu_cred) {
        pr_err("prepare cred failed!\n");
        ret = -ENOMEM;
        goto err_prepare_creds;
    }

    ksu_feature_init();

    ksu_supercalls_init();

    ksu_syscall_hook_manager_init();

    ksu_allowlist_init();

    ret = ksu_pkg_observer_workqueue_init();
    if (ret) {
        pr_err("pkg observer workqueue init failed: %d\n", ret);
        goto err_pkg_observer_workqueue;
    }

    ksu_throne_tracker_init();

    ret = ksu_ksud_init();
    if (ret) {
        pr_err("ksud init failed: %d\n", ret);
        goto err_ksud;
    }

    ksu_file_wrapper_init();

#ifdef MODULE
#ifndef CONFIG_KSU_DEBUG
    ksu_module_prev = THIS_MODULE->list.prev;
    kobject_del(&THIS_MODULE->mkobj.kobj);
    mutex_lock(&module_mutex);
    list_del_rcu(&THIS_MODULE->list);
    mutex_unlock(&module_mutex);
    synchronize_rcu();
#endif
#endif
    return 0;

err_ksud:
    atomic_set(&ksu_module_shutting_down, 1);
    ksu_throne_tracker_exit();
    ksu_pkg_observer_workqueue_exit();
    goto err_allowlist;
err_pkg_observer_workqueue:
    atomic_set(&ksu_module_shutting_down, 1);
err_allowlist:
    ksu_allowlist_exit();
    ksu_syscall_hook_manager_exit();
    ksu_supercalls_exit();
    ksu_feature_exit();
    put_cred(ksu_cred);
err_prepare_creds:
    return ret;
}

void kernelsu_exit(void)
{
    atomic_set(&ksu_module_shutting_down, 1);

    ksu_ksud_exit();

    ksu_observer_exit();

    ksu_pkg_observer_workqueue_exit();

    ksu_throne_tracker_exit();

    ksu_syscall_hook_manager_exit();

    ksu_allowlist_exit();

    ksu_supercalls_exit();

    ksu_feature_exit();

    if (ksu_cred) {
        put_cred(ksu_cred);
    }

#ifdef MODULE
#ifndef CONFIG_KSU_DEBUG
    mutex_lock(&module_mutex);
    list_add_rcu(&THIS_MODULE->list, ksu_module_prev);
    mutex_unlock(&module_mutex);
#endif
#endif
}

module_init(kernelsu_init);
module_exit(kernelsu_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("");
MODULE_DESCRIPTION("Kernel Module");
#if LINUX_VERSION_CODE >= KERNEL_VERSION(6, 13, 0)
MODULE_IMPORT_NS("VFS_internal_I_am_really_a_filesystem_and_am_NOT_a_driver");
#else
MODULE_IMPORT_NS(VFS_internal_I_am_really_a_filesystem_and_am_NOT_a_driver);
#endif
_NOT_a_driver);
#endif
