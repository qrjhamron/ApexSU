// SPDX-License-Identifier: GPL-2.0
#include <linux/err.h>
#include <linux/cred.h>
#include <linux/compiler.h>
#include <linux/fs.h>
#include <linux/list.h>
#include <linux/rwlock.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/types.h>
#include <linux/version.h>
#if IS_ENABLED(CONFIG_KUNIT)
#include <kunit/test.h>
#endif

#include "allowlist.h"
#include "apk_sign.h"
#include "klog.h" // IWYU pragma: keep
#include "manager.h"
#include "throne_tracker.h"

uid_t ksu_manager_appid = KSU_INVALID_APPID;
bool ksu_manager_verified;
char ksu_manager_package[KSU_MAX_PACKAGE_NAME];
static DEFINE_RWLOCK(manager_identity_lock);
static DEFINE_MUTEX(apk_path_hash_lock);
static DEFINE_MUTEX(track_throne_lock);
static u64 manager_scan_epoch;
static u64 manager_verified_epoch;

#define SYSTEM_PACKAGES_LIST_PATH "/data/system/packages.list"

struct uid_data {
    struct list_head list;
    u32 uid;
    char package[KSU_MAX_PACKAGE_NAME];
};

static int parse_packages_list_row(char *buf, ssize_t count,
                                   struct uid_data *out)
{
    char *tmp;
    char *package;
    char *uid;
    char *newline;
    ssize_t copied;
    u32 parsed_uid;
    size_t term_pos;

    if (!buf || !out)
        return -EINVAL;

    if (count < 0)
        return (int)count;

    if (count == 0)
        return -EINVAL;

    if (count >= KSU_MAX_PACKAGE_NAME)
        return -ENAMETOOLONG;

    term_pos = (count < KSU_MAX_PACKAGE_NAME) ? (size_t)count :
                                                (KSU_MAX_PACKAGE_NAME - 1);
    buf[term_pos] = '\0';

    newline = memchr(buf, '\n', count);
    if (!newline)
        return -EINVAL;
    *newline = '\0';

    tmp = buf;
    package = strsep(&tmp, " ");
    uid = strsep(&tmp, " ");
    if (!package || !uid || !package[0] || !uid[0])
        return -EINVAL;

    if (strnlen(package, KSU_MAX_PACKAGE_NAME) >= KSU_MAX_PACKAGE_NAME)
        return -ENAMETOOLONG;

    if (kstrtou32(uid, 10, &parsed_uid))
        return -EINVAL;

    copied = strscpy(out->package, package, sizeof(out->package));
    if (copied < 0)
        return -ENAMETOOLONG;

    out->uid = parsed_uid;
    return 0;
}

static ssize_t packages_list_line_len(loff_t line_start, loff_t line_end)
{
    loff_t len;

    if (line_end <= line_start)
        return -EINVAL;

    len = line_end - line_start;
    if (len >= KSU_MAX_PACKAGE_NAME)
        return -ENAMETOOLONG;

    return (ssize_t)len;
}

bool ksu_is_manager_appid_valid(void)
{
    bool valid;

    read_lock(&manager_identity_lock);
    valid = ksu_manager_verified && ksu_manager_appid != KSU_INVALID_APPID;
    read_unlock(&manager_identity_lock);

    return valid;
}

bool ksu_is_manager_identity_fresh(void)
{
    bool fresh;

    read_lock(&manager_identity_lock);
    fresh = ksu_manager_verified && ksu_manager_appid != KSU_INVALID_APPID &&
            manager_verified_epoch == manager_scan_epoch;
    read_unlock(&manager_identity_lock);

    return fresh;
}

bool is_manager(void)
{
    bool matched;

    read_lock(&manager_identity_lock);
    matched = ksu_manager_verified && ksu_manager_appid != KSU_INVALID_APPID &&
              manager_verified_epoch == manager_scan_epoch &&
              ksu_manager_appid == current_uid().val % PER_USER_RANGE;
    read_unlock(&manager_identity_lock);

    return unlikely(matched);
}

bool is_uid_manager(uid_t uid)
{
    bool matched;

    read_lock(&manager_identity_lock);
    matched = ksu_manager_verified && ksu_manager_appid != KSU_INVALID_APPID &&
              manager_verified_epoch == manager_scan_epoch &&
              ksu_manager_appid == uid % PER_USER_RANGE;
    read_unlock(&manager_identity_lock);

    return unlikely(matched);
}

uid_t ksu_get_manager_appid(void)
{
    uid_t appid;

    read_lock(&manager_identity_lock);
    appid = ksu_manager_verified ? ksu_manager_appid : KSU_INVALID_APPID;
    read_unlock(&manager_identity_lock);

    return appid;
}

void ksu_set_manager_identity(uid_t appid, const char *package)
{
    if (!package || !package[0]) {
        pr_warn("refusing to set manager identity without package\n");
        return;
    }

    write_lock(&manager_identity_lock);
    strscpy(ksu_manager_package, package, sizeof(ksu_manager_package));
    ksu_manager_appid = appid;
    ksu_manager_verified = true;
    manager_verified_epoch = manager_scan_epoch;
    write_unlock(&manager_identity_lock);

    pr_info("manager identity verified: %s(appid=%d, epoch=%llu)\n", package,
            appid, (unsigned long long)manager_verified_epoch);
}

void ksu_set_manager_appid(uid_t appid)
{
#ifdef CONFIG_KSU_DEBUG
    ksu_set_manager_identity(appid, "debug.manager");
#else
    pr_warn(
        "ignoring manager appid update without verified package identity\n");
#endif
}

void ksu_invalidate_manager_uid(void)
{
    ksu_invalidate_manager_uid_reason("unspecified");
}

void ksu_invalidate_manager_uid_reason(const char *reason)
{
    write_lock(&manager_identity_lock);
    ksu_manager_verified = false;
    ksu_manager_appid = KSU_INVALID_APPID;
    ksu_manager_package[0] = '\0';
    manager_verified_epoch = 0;
    write_unlock(&manager_identity_lock);

    pr_warn("manager identity invalidated: %s\n",
            reason ? reason : "unknown reason");
}

void ksu_mark_manager_identity_refresh_start(const char *reason)
{
    write_lock(&manager_identity_lock);
    manager_scan_epoch++;
    write_unlock(&manager_identity_lock);

    pr_info("manager identity refresh start (epoch=%llu): %s\n",
            (unsigned long long)manager_scan_epoch, reason ? reason : "scan");
}

bool ksu_manager_identity_matches(uid_t appid, const char *package)
{
    bool matched;

    if (!package || !ksu_is_manager_appid_valid())
        return false;

    read_lock(&manager_identity_lock);
    matched =
        ksu_manager_verified && ksu_manager_appid == appid &&
        strncmp(ksu_manager_package, package, sizeof(ksu_manager_package)) == 0;
    read_unlock(&manager_identity_lock);

    return matched;
}

static void crown_manager(const char *apk, struct list_head *uid_data)
{
    char pkg[KSU_MAX_PACKAGE_NAME];
    if (get_pkg_from_apk_path(pkg, apk) < 0) {
        pr_err("Failed to get package name from apk path: %s\n", apk);
        return;
    }

    pr_info("manager pkg: %s\n", pkg);

    struct list_head *list = (struct list_head *)uid_data;
    struct uid_data *np;

    list_for_each_entry (np, list, list) {
        if (strncmp(np->package, pkg, KSU_MAX_PACKAGE_NAME) == 0) {
            pr_info("Crowning manager: %s(uid=%d)\n", pkg, np->uid);
            ksu_set_manager_identity(np->uid, pkg);
            break;
        }
    }
}

#define DATA_PATH_LEN 384 // 384 is enough for /data/app/<package>/base.apk

struct data_path {
    char dirpath[DATA_PATH_LEN];
    int depth;
    struct list_head list;
};

struct apk_path_hash {
    unsigned int hash;
    bool exists;
    struct list_head list;
};

struct apk_candidate {
    char apkpath[DATA_PATH_LEN];
    unsigned int hash;
    struct list_head list;
};

static struct list_head apk_path_hash_list = LIST_HEAD_INIT(apk_path_hash_list);

static void clear_apk_path_hash_cache_locked(void)
{
    struct apk_path_hash *pos, *n;

    list_for_each_entry_safe (pos, n, &apk_path_hash_list, list) {
        list_del(&pos->list);
        kfree(pos);
    }
}

static void clear_apk_path_hash_cache(void)
{
    mutex_lock(&apk_path_hash_lock);
    clear_apk_path_hash_cache_locked();
    mutex_unlock(&apk_path_hash_lock);
}

struct my_dir_context {
    struct dir_context ctx;
    struct list_head *data_path_list;
    struct list_head *apk_candidate_list;
    char *parent_dir;
    void *private_data;
    int depth;
    int *stop;
};
// https://docs.kernel.org/filesystems/porting.html
// filldir_t (readdir callbacks) calling conventions have changed. Instead of returning 0 or -E... it returns bool now. false means "no more" (as -E... used to) and true - "keep going" (as 0 in old calling conventions). Rationale: callers never looked at specific -E... values anyway. -> iterate_shared() instances require no changes at all, all filldir_t ones in the tree converted.
#if LINUX_VERSION_CODE >= KERNEL_VERSION(6, 1, 0)
#define FILLDIR_RETURN_TYPE bool
#define FILLDIR_ACTOR_CONTINUE true
#define FILLDIR_ACTOR_STOP false
#else
#define FILLDIR_RETURN_TYPE int
#define FILLDIR_ACTOR_CONTINUE 0
#define FILLDIR_ACTOR_STOP -EINVAL
#endif
extern int is_manager_apk(char *path);
FILLDIR_RETURN_TYPE my_actor(struct dir_context *ctx, const char *name,
                             int namelen, loff_t off, u64 ino,
                             unsigned int d_type)
{
    struct my_dir_context *my_ctx =
        container_of(ctx, struct my_dir_context, ctx);
    char dirpath[DATA_PATH_LEN];

    if (!my_ctx) {
        pr_err("Invalid context\n");
        return FILLDIR_ACTOR_STOP;
    }
    if (my_ctx->stop && *my_ctx->stop) {
        pr_info("Stop searching\n");
        return FILLDIR_ACTOR_STOP;
    }

    if (!strncmp(name, "..", namelen) || !strncmp(name, ".", namelen))
        return FILLDIR_ACTOR_CONTINUE; // Skip "." and ".."

    if (d_type == DT_DIR && namelen >= 8 && !strncmp(name, "vmdl", 4) &&
        !strncmp(name + namelen - 4, ".tmp", 4)) {
        pr_info("Skipping directory: %.*s\n", namelen, name);
        return FILLDIR_ACTOR_CONTINUE; // Skip staging package
    }

    if (snprintf(dirpath, DATA_PATH_LEN, "%s/%.*s", my_ctx->parent_dir, namelen,
                 name) >= DATA_PATH_LEN) {
        pr_err("Path too long: %s/%.*s\n", my_ctx->parent_dir, namelen, name);
        return FILLDIR_ACTOR_CONTINUE;
    }

    if (d_type == DT_DIR && my_ctx->depth > 0 &&
        (my_ctx->stop && !*my_ctx->stop)) {
        struct data_path *data = kzalloc(sizeof(struct data_path), GFP_ATOMIC);

        if (!data) {
            pr_err("Failed to allocate memory for %s\n", dirpath);
            return FILLDIR_ACTOR_CONTINUE;
        }

        strscpy(data->dirpath, dirpath, DATA_PATH_LEN);
        data->depth = my_ctx->depth - 1;
        list_add_tail(&data->list, my_ctx->data_path_list);
    } else {
        if ((namelen == 8) && (strncmp(name, "base.apk", namelen) == 0)) {
            struct apk_path_hash *pos;
            struct apk_candidate *candidate;
            unsigned int hash = full_name_hash(NULL, dirpath, strlen(dirpath));
            bool cached = false;

            mutex_lock(&apk_path_hash_lock);
            list_for_each_entry (pos, &apk_path_hash_list, list) {
                if (hash == pos->hash) {
                    pos->exists = true;
                    cached = true;
                    break;
                }
            }
            mutex_unlock(&apk_path_hash_lock);
            if (cached)
                return FILLDIR_ACTOR_CONTINUE;

            candidate = kzalloc(sizeof(*candidate), GFP_ATOMIC);
            if (!candidate) {
                pr_warn("Failed to allocate apk_candidate\n");
                return FILLDIR_ACTOR_CONTINUE;
            }

            strscpy(candidate->apkpath, dirpath, sizeof(candidate->apkpath));
            candidate->hash = hash;
            list_add_tail(&candidate->list, my_ctx->apk_candidate_list);
        }
    }

    return FILLDIR_ACTOR_CONTINUE;
}

static void process_apk_candidates(struct list_head *apk_candidate_list,
                                   struct list_head *uid_data, int *stop)
{
    struct apk_candidate *candidate;
    struct apk_candidate *n;

    list_for_each_entry_safe (candidate, n, apk_candidate_list, list) {
        int is_manager;

        list_del(&candidate->list);

        if (stop && *stop) {
            kfree(candidate);
            continue;
        }

        is_manager = is_manager_apk(candidate->apkpath);
        pr_info("Found new base.apk at path: %s, is_manager: %d\n",
                candidate->apkpath, is_manager);
        if (is_manager == 1) {
            crown_manager(candidate->apkpath, uid_data);
            if (stop)
                *stop = 1;

            // Manager found, clear APK cache list
            clear_apk_path_hash_cache();
        } else if (is_manager == 0) {
            // Definitely not manager — cache path to skip next time
            struct apk_path_hash *apk_data =
                kzalloc(sizeof(struct apk_path_hash), GFP_KERNEL);
            if (!apk_data) {
                pr_warn("Failed to allocate apk_path_hash\n");
                kfree(candidate);
                continue;
            }
            apk_data->hash = candidate->hash;
            apk_data->exists = true;
            mutex_lock(&apk_path_hash_lock);
            list_add_tail(&apk_data->list, &apk_path_hash_list);
            mutex_unlock(&apk_path_hash_lock);
        }
        // is_manager < 0: error reading APK, don't cache — retry next time
        kfree(candidate);
    }
}

void search_manager(const char *path, int depth, struct list_head *uid_data)
{
    int i, stop = 0;
    struct list_head data_path_list;
    struct list_head apk_candidate_list;
    INIT_LIST_HEAD(&data_path_list);
    INIT_LIST_HEAD(&apk_candidate_list);
    unsigned long data_app_magic = 0;

    // Initialize APK cache list
    struct apk_path_hash *pos, *n;
    mutex_lock(&apk_path_hash_lock);
    list_for_each_entry (pos, &apk_path_hash_list, list) {
        pos->exists = false;
    }
    mutex_unlock(&apk_path_hash_lock);

    // First depth
    struct data_path data;
    strscpy(data.dirpath, path, DATA_PATH_LEN);
    data.depth = depth;
    list_add_tail(&data.list, &data_path_list);

    for (i = depth; i >= 0; i--) {
        struct data_path *pos, *n;

        list_for_each_entry_safe (pos, n, &data_path_list, list) {
            struct my_dir_context ctx = { .ctx.actor = my_actor,
                                          .data_path_list = &data_path_list,
                                          .apk_candidate_list =
                                              &apk_candidate_list,
                                          .parent_dir = pos->dirpath,
                                          .private_data = uid_data,
                                          .depth = pos->depth,
                                          .stop = &stop };
            struct file *file;

            if (!stop) {
                file = filp_open(pos->dirpath, O_RDONLY | O_NOFOLLOW, 0);
                if (IS_ERR(file)) {
                    pr_err("Failed to open directory: %s, err: %ld\n",
                           pos->dirpath, PTR_ERR(file));
                    goto skip_iterate;
                }

                // grab magic on first folder, which is /data/app
                if (!data_app_magic) {
                    if (file->f_inode->i_sb->s_magic) {
                        data_app_magic = file->f_inode->i_sb->s_magic;
                        pr_info("%s: dir: %s got magic! 0x%lx\n", __func__,
                                pos->dirpath, data_app_magic);
                    } else {
                        filp_close(file, NULL);
                        goto skip_iterate;
                    }
                }

                if (file->f_inode->i_sb->s_magic != data_app_magic) {
                    pr_info("%s: skip: %s magic: 0x%lx expected: 0x%lx\n",
                            __func__, pos->dirpath,
                            file->f_inode->i_sb->s_magic, data_app_magic);
                    filp_close(file, NULL);
                    goto skip_iterate;
                }

                iterate_dir(file, &ctx.ctx);
                filp_close(file, NULL);
                process_apk_candidates(&apk_candidate_list, uid_data, &stop);
            }
        skip_iterate:
            list_del(&pos->list);
            if (pos != &data)
                kfree(pos);
        }
    }

    process_apk_candidates(&apk_candidate_list, uid_data, &stop);

    // Remove stale cached APK entries
    mutex_lock(&apk_path_hash_lock);
    list_for_each_entry_safe (pos, n, &apk_path_hash_list, list) {
        if (!pos->exists) {
            list_del(&pos->list);
            kfree(pos);
        }
    }
    mutex_unlock(&apk_path_hash_lock);
}

static bool is_uid_exist(uid_t uid, char *package, void *data)
{
    struct list_head *list = (struct list_head *)data;
    struct uid_data *np;

    bool exist = false;
    list_for_each_entry (np, list, list) {
        if (np->uid == uid % PER_USER_RANGE &&
            strncmp(np->package, package, KSU_MAX_PACKAGE_NAME) == 0) {
            exist = true;
            break;
        }
    }
    return exist;
}

static bool manager_identity_exists_in_packages(struct list_head *uid_list)
{
    struct uid_data *np;
    uid_t manager_appid = ksu_get_manager_appid();
    bool matched = false;
    bool duplicate = false;

    if (manager_appid == KSU_INVALID_APPID)
        return false;

    list_for_each_entry (np, uid_list, list) {
        if (np->uid != manager_appid)
            continue;

        if (ksu_manager_identity_matches(np->uid, np->package)) {
            if (matched)
                duplicate = true;
            matched = true;
            continue;
        }

        pr_warn(
            "manager appid %d also belongs to non-manager package %s; invalidating\n",
            np->uid, np->package);
        return false;
    }

    if (duplicate) {
        pr_warn("manager appid %d appears multiple times; invalidating\n",
                manager_appid);
        return false;
    }

    if (!matched)
        pr_info("manager appid %d is not present in packages.list\n",
                manager_appid);

    return matched;
}

void track_throne(bool prune_only)
{
    struct uid_data *np;
    struct uid_data *n;
    bool refresh_failed = false;
    const char *refresh_fail_reason = NULL;

    /*
     * Serialize scans to avoid re-entrant traversal from fsnotify and boot
     * events. APK hash list itself uses a dedicated mutex to avoid lockless
     * mutation, and we keep that lock out of slow file IO paths.
     */
    mutex_lock(&track_throne_lock);
    if (!prune_only)
        ksu_mark_manager_identity_refresh_start("packages.list scan");

    struct file *fp = filp_open(SYSTEM_PACKAGES_LIST_PATH, O_RDONLY, 0);
    if (IS_ERR(fp)) {
        pr_err("%s: open " SYSTEM_PACKAGES_LIST_PATH " failed: %ld\n", __func__,
               PTR_ERR(fp));
        if (!prune_only) {
            refresh_failed = true;
            refresh_fail_reason = "packages.list open failed";
        }
        mutex_unlock(&track_throne_lock);
        if (refresh_failed)
            ksu_invalidate_manager_uid_reason(refresh_fail_reason);
        return;
    }

    struct list_head uid_list;
    INIT_LIST_HEAD(&uid_list);

    char chr = 0;
    loff_t pos = 0;
    loff_t line_start = 0;
    char buf[KSU_MAX_PACKAGE_NAME];
    bool parse_failed = false;
    int parse_ret;
    for (;;) {
        ssize_t count = kernel_read(fp, &chr, sizeof(chr), &pos);
        if (count < 0) {
            pr_err("update_uid: failed to scan packages.list: %zd\n", count);
            parse_failed = true;
            break;
        }
        if (count != sizeof(chr))
            break;
        if (chr != '\n')
            continue;

        count = packages_list_line_len(line_start, pos);
        if (count < 0) {
            if (count == -ENAMETOOLONG)
                pr_err("update_uid: packages.list row too long/incomplete\n");
            else
                pr_err("update_uid: invalid packages.list line length: %zd\n",
                       count);
            parse_failed = true;
            break;
        }

        {
            loff_t row_pos = line_start;
            ssize_t want = count;

            count = kernel_read(fp, buf, want, &row_pos);
            if (count >= 0 && count != want) {
                pr_err("update_uid: short packages.list line read: %zd/%zd\n",
                       count, want);
                parse_failed = true;
                break;
            }
        }
        if (count < 0) {
            pr_err("update_uid: failed to read packages.list line: %zd\n",
                   count);
            parse_failed = true;
            break;
        }
        if (count == 0) {
            pr_err("update_uid: empty packages.list line\n");
            parse_failed = true;
            break;
        }

        struct uid_data *data = kzalloc(sizeof(struct uid_data), GFP_ATOMIC);
        if (!data) {
            parse_failed = true;
            filp_close(fp, 0);
            goto out;
        }

        parse_ret = parse_packages_list_row(buf, count, data);
        if (parse_ret) {
            kfree(data);
            if (parse_ret == -ENAMETOOLONG)
                pr_err("update_uid: packages.list row too long/incomplete\n");
            else
                pr_err("update_uid: malformed packages.list row (err=%d)\n",
                       parse_ret);
            parse_failed = true;
            break;
        }
        list_add_tail(&data->list, &uid_list);
        // reset line start
        line_start = pos;
    }
    if (!parse_failed && pos != line_start) {
        pr_err(
            "update_uid: trailing unterminated packages.list row rejected\n");
        parse_failed = true;
    }
    filp_close(fp, 0);
    if (parse_failed) {
        if (!prune_only) {
            refresh_failed = true;
            refresh_fail_reason = "packages.list parse failed";
        }
        goto out;
    }

    if (prune_only)
        goto prune;

    if (ksu_is_manager_appid_valid()) {
        if (!manager_identity_exists_in_packages(&uid_list)) {
            ksu_invalidate_manager_uid_reason(
                "manager appid mismatch in packages.list");
        } else {
            /*
             * The kernel scans raw /data/app APK files, so package-list
             * identity alone is not enough. Re-scan and re-run manager APK
             * signature/cert checks before manager-only ioctls remain enabled.
             */
            pr_info("Refreshing verified manager identity\n");
            ksu_invalidate_manager_uid_reason(
                "forcing signature re-verification");
            clear_apk_path_hash_cache();
            search_manager("/data/app", 2, &uid_list);
        }
    } else {
        pr_info("Searching manager...\n");
        clear_apk_path_hash_cache();
        search_manager("/data/app", 2, &uid_list);
    }
    if (!ksu_is_manager_appid_valid())
        pr_warn("No verified manager identity after throne tracking\n");
    pr_info("Search manager finished\n");

prune:
    // then prune the allowlist
    ksu_prune_allowlist(is_uid_exist, &uid_list);
out:
    // free uid_list
    list_for_each_entry_safe (np, n, &uid_list, list) {
        list_del(&np->list);
        kfree(np);
    }
    mutex_unlock(&track_throne_lock);
    if (refresh_failed)
        ksu_invalidate_manager_uid_reason(refresh_fail_reason);
}

void ksu_throne_tracker_init()
{
    // nothing to do
}

void ksu_throne_tracker_exit()
{
    // nothing to do
}

#if IS_ENABLED(CONFIG_KUNIT)
static void reset_manager_identity_state(void)
{
    write_lock(&manager_identity_lock);
    ksu_manager_appid = KSU_INVALID_APPID;
    ksu_manager_verified = false;
    ksu_manager_package[0] = '\0';
    manager_scan_epoch = 0;
    manager_verified_epoch = 0;
    write_unlock(&manager_identity_lock);
}

static void manager_identity_fresh_after_verified_set_test(struct kunit *test)
{
    reset_manager_identity_state();
    ksu_mark_manager_identity_refresh_start("kunit");
    ksu_set_manager_identity(12345, "me.weishu.kernelsu");

    KUNIT_EXPECT_TRUE(test, ksu_is_manager_identity_fresh());
    KUNIT_EXPECT_TRUE(test, is_uid_manager(12345));
}

static void manager_identity_stale_after_refresh_start_test(struct kunit *test)
{
    reset_manager_identity_state();
    ksu_mark_manager_identity_refresh_start("kunit");
    ksu_set_manager_identity(12345, "me.weishu.kernelsu");

    ksu_mark_manager_identity_refresh_start("kunit-stale");
    KUNIT_EXPECT_FALSE(test, ksu_is_manager_identity_fresh());
    KUNIT_EXPECT_FALSE(test, is_uid_manager(12345));
}

static void
manager_identity_removed_or_wrong_package_denied_test(struct kunit *test)
{
    struct list_head uid_list;
    struct uid_data row = {};

    reset_manager_identity_state();
    ksu_mark_manager_identity_refresh_start("kunit");
    ksu_set_manager_identity(12345, "me.weishu.kernelsu");

    INIT_LIST_HEAD(&uid_list);
    row.uid = 12345;
    strscpy(row.package, "com.example.wrong", sizeof(row.package));
    list_add_tail(&row.list, &uid_list);

    KUNIT_EXPECT_FALSE(test, manager_identity_exists_in_packages(&uid_list));
    list_del(&row.list);
}

static struct kunit_case throne_tracker_manager_identity_cases[] = {
    KUNIT_CASE(manager_identity_fresh_after_verified_set_test),
    KUNIT_CASE(manager_identity_stale_after_refresh_start_test),
    KUNIT_CASE(manager_identity_removed_or_wrong_package_denied_test),
    {}
};

static struct kunit_suite throne_tracker_manager_identity_suite = {
    .name = "ksu_throne_tracker_manager_identity",
    .test_cases = throne_tracker_manager_identity_cases,
};

kunit_test_suite(throne_tracker_manager_identity_suite);

static void parse_row_rejects_no_space_line_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[] = "nospace\n";

    KUNIT_EXPECT_LT(test, parse_packages_list_row(line, strlen(line), &out), 0);
}

static void parse_row_rejects_exact_buffer_size_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[KSU_MAX_PACKAGE_NAME];

    memset(line, 'a', sizeof(line));
    KUNIT_EXPECT_EQ(test, parse_packages_list_row(line, sizeof(line), &out),
                    -ENAMETOOLONG);
}

static void line_len_uses_detected_line_not_buffer_size_test(struct kunit *test)
{
    KUNIT_EXPECT_EQ(test, packages_list_line_len(10, 42), (ssize_t)32);
}

static void line_len_rejects_exact_buffer_size_test(struct kunit *test)
{
    KUNIT_EXPECT_EQ(test, packages_list_line_len(0, KSU_MAX_PACKAGE_NAME),
                    (ssize_t)-ENAMETOOLONG);
}

static void parse_row_rejects_overlong_package_name_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[KSU_MAX_PACKAGE_NAME + 64];
    size_t i = 0;

    for (; i < KSU_MAX_PACKAGE_NAME; i++)
        line[i] = 'p';
    line[i++] = ' ';
    line[i++] = '1';
    line[i++] = '0';
    line[i++] = '0';
    line[i++] = '0';
    line[i++] = '\n';
    line[i] = '\0';

    KUNIT_EXPECT_EQ(test, parse_packages_list_row(line, strlen(line), &out),
                    -ENAMETOOLONG);
}

static void parse_row_rejects_no_trailing_newline_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[] = "com.example.app 1000";

    KUNIT_EXPECT_LT(test, parse_packages_list_row(line, strlen(line), &out), 0);
}

static void parse_row_rejects_malformed_uid_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[] = "com.example.app not_uid /data/app/example/base.apk\n";

    KUNIT_EXPECT_EQ(test, parse_packages_list_row(line, strlen(line), &out),
                    -EINVAL);
}

static void parse_row_accepts_valid_manager_row_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[] =
        "me.weishu.kernelsu 10234 1 /data/user/0 default:targetSdkVersion=34 none 0 0 1 @null\n";

    KUNIT_EXPECT_EQ(test, parse_packages_list_row(line, strlen(line), &out), 0);
    KUNIT_EXPECT_EQ(test, out.uid, (u32)10234);
    KUNIT_EXPECT_STREQ(test, out.package, "me.weishu.kernelsu");
}

static void
parse_row_accepts_valid_non_manager_same_appid_test(struct kunit *test)
{
    struct uid_data out = {};
    char line[] =
        "com.example.other 10234 1 /data/user/0 default:targetSdkVersion=34 none 0 0 1 @null\n";

    KUNIT_EXPECT_EQ(test, parse_packages_list_row(line, strlen(line), &out), 0);
    KUNIT_EXPECT_EQ(test, out.uid, (u32)10234);
    KUNIT_EXPECT_STREQ(test, out.package, "com.example.other");
}

static struct kunit_case throne_tracker_parse_cases[] = {
    KUNIT_CASE(parse_row_rejects_no_space_line_test),
    KUNIT_CASE(parse_row_rejects_exact_buffer_size_test),
    KUNIT_CASE(line_len_uses_detected_line_not_buffer_size_test),
    KUNIT_CASE(line_len_rejects_exact_buffer_size_test),
    KUNIT_CASE(parse_row_rejects_overlong_package_name_test),
    KUNIT_CASE(parse_row_rejects_no_trailing_newline_test),
    KUNIT_CASE(parse_row_rejects_malformed_uid_test),
    KUNIT_CASE(parse_row_accepts_valid_manager_row_test),
    KUNIT_CASE(parse_row_accepts_valid_non_manager_same_appid_test),
    {}
};

static struct kunit_suite throne_tracker_parse_suite = {
    .name = "ksu_throne_tracker_parse",
    .test_cases = throne_tracker_parse_cases,
};

kunit_test_suite(throne_tracker_parse_suite);
#endif
