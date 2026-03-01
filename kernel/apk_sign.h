// SPDX-License-Identifier: GPL-2.0
#ifndef __KSU_H_APK_V2_SIGN
#define __KSU_H_APK_V2_SIGN

#include <linux/types.h>

int is_manager_apk(char *path);
int get_pkg_from_apk_path(char *pkg, const char *path);

#endif
