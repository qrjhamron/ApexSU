// SPDX-License-Identifier: GPL-2.0
#ifndef __KSU_H_KLOG
#define __KSU_H_KLOG

#include <linux/printk.h>

#ifdef pr_fmt
#undef pr_fmt
#define pr_fmt(fmt) "KernelSU: " fmt
#endif

#ifndef CONFIG_KSU_DEBUG
#undef pr_info
#define pr_info(fmt, ...)                                                      \
    do {                                                                       \
    } while (0)
#undef pr_warn
#define pr_warn(fmt, ...)                                                      \
    do {                                                                       \
    } while (0)
#endif

#endif
