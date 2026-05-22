// SPDX-License-Identifier: GPL-2.0
#ifndef __KSU_H_KSU_MANAGER
#define __KSU_H_KSU_MANAGER

#include <linux/cred.h>
#include <linux/types.h>
#include "allowlist.h"

#define KSU_INVALID_APPID -1

extern uid_t ksu_manager_appid; // DO NOT DIRECT USE
extern bool ksu_manager_verified; // DO NOT DIRECT USE
extern char ksu_manager_package[KSU_MAX_PACKAGE_NAME]; // DO NOT DIRECT USE

bool ksu_is_manager_appid_valid(void);
bool ksu_is_manager_identity_fresh(void);

bool is_manager(void);

bool is_uid_manager(uid_t uid);

uid_t ksu_get_manager_appid(void);

void ksu_set_manager_identity(uid_t appid, const char *package);

void ksu_set_manager_appid(uid_t appid);

void ksu_invalidate_manager_uid(void);
void ksu_invalidate_manager_uid_reason(const char *reason);
void ksu_mark_manager_identity_refresh_start(const char *reason);

bool ksu_manager_identity_matches(uid_t appid, const char *package);

int ksu_observer_init(void);
#endif
