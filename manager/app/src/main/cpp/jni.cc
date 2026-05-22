#include <jni.h>

#include <sys/prctl.h>
#include <linux/capability.h>
#include <pwd.h>

#include <android/log.h>
#include <cstring>

#include "ksu.h"

#define LOG_TAG "KernelSU"
#ifdef NDEBUG
#define LOGD(...) (void)0
#else
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#endif

extern "C"
JNIEXPORT jint JNICALL
Java_com_qrj_apexsu_Natives_getVersion(JNIEnv *env, jobject) {
    return get_version();
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_qrj_apexsu_Natives_getSuperuserCount(JNIEnv *env, jobject) {
    struct ksu_new_get_allow_list_cmd cmd = {
        .count = 0
    };
    bool result = get_allow_list(&cmd);
    return result ? cmd.total_count : 0;
}

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_isSafeMode(JNIEnv *env, jclass clazz) {
    return is_safe_mode();
}

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_isLkmMode(JNIEnv *env, jclass clazz) {
    return is_lkm_mode();
}

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_isManager(JNIEnv *env, jclass clazz) {
    return is_manager();
}

static void fillIntArray(JNIEnv *env, jobject list, int *data, int count) {
    if (!list || !data || count <= 0) {
        return;
    }
    auto cls = env->GetObjectClass(list);
    auto add = env->GetMethodID(cls, "add", "(Ljava/lang/Object;)Z");
    auto integerCls = env->FindClass("java/lang/Integer");
    auto constructor = env->GetMethodID(integerCls, "<init>", "(I)V");
    for (int i = 0; i < count; ++i) {
        auto integer = env->NewObject(integerCls, constructor, data[i]);
        env->CallBooleanMethod(list, add, integer);
        env->DeleteLocalRef(integer);
    }
    env->DeleteLocalRef(integerCls);
    env->DeleteLocalRef(cls);
}

static void addIntToList(JNIEnv *env, jobject list, int ele) {
    if (!list) {
        return;
    }
    auto cls = env->GetObjectClass(list);
    auto add = env->GetMethodID(cls, "add", "(Ljava/lang/Object;)Z");
    auto integerCls = env->FindClass("java/lang/Integer");
    auto constructor = env->GetMethodID(integerCls, "<init>", "(I)V");
    auto integer = env->NewObject(integerCls, constructor, ele);
    env->CallBooleanMethod(list, add, integer);
    env->DeleteLocalRef(integer);
    env->DeleteLocalRef(integerCls);
    env->DeleteLocalRef(cls);
}

static uint64_t capListToBits(JNIEnv *env, jobject list) {
    if (!list) {
        return 0;
    }
    auto cls = env->GetObjectClass(list);
    auto get = env->GetMethodID(cls, "get", "(I)Ljava/lang/Object;");
    auto size = env->GetMethodID(cls, "size", "()I");
    auto listSize = env->CallIntMethod(list, size);
    auto integerCls = env->FindClass("java/lang/Integer");
    auto intValue = env->GetMethodID(integerCls, "intValue", "()I");
    uint64_t result = 0;
    for (int i = 0; i < listSize; ++i) {
        auto integer = env->CallObjectMethod(list, get, i);
        if (!integer) {
            continue;
        }
        int data = env->CallIntMethod(integer, intValue);
        env->DeleteLocalRef(integer);

        if (cap_valid(data)) {
            result |= (1ULL << data);
        }
    }

    env->DeleteLocalRef(integerCls);
    env->DeleteLocalRef(cls);
    return result;
}

static int getListSize(JNIEnv *env, jobject list) {
    if (!list) {
        return 0;
    }
    auto cls = env->GetObjectClass(list);
    auto size = env->GetMethodID(cls, "size", "()I");
    int result = env->CallIntMethod(list, size);
    env->DeleteLocalRef(cls);
    return result;
}

static void fillArrayWithList(JNIEnv *env, jobject list, int *data, int count) {
    if (!list || !data || count <= 0) {
        return;
    }
    auto cls = env->GetObjectClass(list);
    auto get = env->GetMethodID(cls, "get", "(I)Ljava/lang/Object;");
    auto integerCls = env->FindClass("java/lang/Integer");
    auto intValue = env->GetMethodID(integerCls, "intValue", "()I");
    for (int i = 0; i < count; ++i) {
        auto integer = env->CallObjectMethod(list, get, i);
        if (!integer) {
            continue;
        }
        data[i] = env->CallIntMethod(integer, intValue);
        env->DeleteLocalRef(integer);
    }
    env->DeleteLocalRef(integerCls);
    env->DeleteLocalRef(cls);
}

template <size_t N>
static bool copyJStringToFixed(JNIEnv *env, jstring src, char (&dst)[N]) {
    if (!src) {
        return false;
    }
    auto len = env->GetStringUTFLength(src);
    if (len < 0 || static_cast<size_t>(len) >= N) {
        return false;
    }
    auto chars = env->GetStringUTFChars(src, nullptr);
    if (!chars) {
        return false;
    }
    strncpy(dst, chars, N - 1);
    dst[N - 1] = '\0';
    env->ReleaseStringUTFChars(src, chars);
    return true;
}

extern "C"
JNIEXPORT jobject JNICALL
Java_com_qrj_apexsu_Natives_getAppProfile(JNIEnv *env, jobject, jstring pkg, jint uid) {
    if (!pkg) {
        return nullptr;
    }

    p_key_t key = {};
    if (!copyJStringToFixed(env, pkg, key)) {
        return nullptr;
    }

    app_profile profile = {};
    profile.version = KSU_APP_PROFILE_VER;

    strcpy(profile.key, key);
    profile.current_uid = uid;

    bool useDefaultProfile = get_app_profile(&profile) != 0;

    auto cls = env->FindClass("com/qrj/apexsu/Natives$Profile");
    auto constructor = env->GetMethodID(cls, "<init>", "()V");
    auto obj = env->NewObject(cls, constructor);
    auto keyField = env->GetFieldID(cls, "name", "Ljava/lang/String;");
    auto currentUidField = env->GetFieldID(cls, "currentUid", "I");
    auto allowSuField = env->GetFieldID(cls, "allowSu", "Z");

    auto rootUseDefaultField = env->GetFieldID(cls, "rootUseDefault", "Z");
    auto rootTemplateField = env->GetFieldID(cls, "rootTemplate", "Ljava/lang/String;");

    auto uidField = env->GetFieldID(cls, "uid", "I");
    auto gidField = env->GetFieldID(cls, "gid", "I");
    auto groupsField = env->GetFieldID(cls, "groups", "Ljava/util/List;");
    auto capabilitiesField = env->GetFieldID(cls, "capabilities", "Ljava/util/List;");
    auto domainField = env->GetFieldID(cls, "context", "Ljava/lang/String;");
    auto namespacesField = env->GetFieldID(cls, "namespace", "I");

    auto nonRootUseDefaultField = env->GetFieldID(cls, "nonRootUseDefault", "Z");
    auto umountModulesField = env->GetFieldID(cls, "umountModules", "Z");

    auto profileKey = env->NewStringUTF(profile.key);
    env->SetObjectField(obj, keyField, profileKey);
    env->DeleteLocalRef(profileKey);
    env->SetIntField(obj, currentUidField, profile.current_uid);

    if (useDefaultProfile) {
        // no profile found, so just use default profile:
        // don't allow root and use default profile!
        LOGD("use default profile for: %s, %d", key, uid);

        // allow_su = false
        // non root use default = true
        env->SetBooleanField(obj, allowSuField, false);
        env->SetBooleanField(obj, nonRootUseDefaultField, true);

        env->DeleteLocalRef(cls);
        return obj;
    }

    auto allowSu = profile.allow_su;

    if (allowSu) {
        env->SetBooleanField(obj, rootUseDefaultField, (jboolean) profile.rp_config.use_default);
        if (strlen(profile.rp_config.template_name) > 0) {
            auto templateName = env->NewStringUTF(profile.rp_config.template_name);
            env->SetObjectField(obj, rootTemplateField, templateName);
            env->DeleteLocalRef(templateName);
        }

        env->SetIntField(obj, uidField, profile.rp_config.profile.uid);
        env->SetIntField(obj, gidField, profile.rp_config.profile.gid);

        jobject groupList = env->GetObjectField(obj, groupsField);
        int groupCount = profile.rp_config.profile.groups_count;
        if (groupCount > KSU_MAX_GROUPS) {
            LOGD("kernel group count too large: %d???", groupCount);
            groupCount = KSU_MAX_GROUPS;
        }
        fillIntArray(env, groupList, profile.rp_config.profile.groups, groupCount);
        env->DeleteLocalRef(groupList);

        jobject capList = env->GetObjectField(obj, capabilitiesField);
        for (int i = 0; i <= CAP_LAST_CAP; i++) {
            if (profile.rp_config.profile.capabilities.effective & (1ULL << i)) {
                addIntToList(env, capList, i);
            }
        }
        env->DeleteLocalRef(capList);

        auto domain = env->NewStringUTF(profile.rp_config.profile.selinux_domain);
        env->SetObjectField(obj, domainField, domain);
        env->DeleteLocalRef(domain);
        env->SetIntField(obj, namespacesField, profile.rp_config.profile.namespaces);
        env->SetBooleanField(obj, allowSuField, profile.allow_su);
    } else {
        env->SetBooleanField(obj, nonRootUseDefaultField,
                (jboolean) profile.nrp_config.use_default);
        env->SetBooleanField(obj, umountModulesField, profile.nrp_config.profile.umount_modules);
    }

    env->DeleteLocalRef(cls);
    return obj;
}

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_setAppProfile(JNIEnv *env, jobject clazz, jobject profile) {
    if (!profile) {
        return false;
    }

    auto cls = env->FindClass("com/qrj/apexsu/Natives$Profile");
    if (!cls) {
        return false;
    }
    bool success = false;

    auto keyField = env->GetFieldID(cls, "name", "Ljava/lang/String;");
    auto currentUidField = env->GetFieldID(cls, "currentUid", "I");
    auto allowSuField = env->GetFieldID(cls, "allowSu", "Z");

    auto rootUseDefaultField = env->GetFieldID(cls, "rootUseDefault", "Z");
    auto rootTemplateField = env->GetFieldID(cls, "rootTemplate", "Ljava/lang/String;");

    auto uidField = env->GetFieldID(cls, "uid", "I");
    auto gidField = env->GetFieldID(cls, "gid", "I");
    auto groupsField = env->GetFieldID(cls, "groups", "Ljava/util/List;");
    auto capabilitiesField = env->GetFieldID(cls, "capabilities", "Ljava/util/List;");
    auto domainField = env->GetFieldID(cls, "context", "Ljava/lang/String;");
    auto namespacesField = env->GetFieldID(cls, "namespace", "I");

    auto nonRootUseDefaultField = env->GetFieldID(cls, "nonRootUseDefault", "Z");
    auto umountModulesField = env->GetFieldID(cls, "umountModules", "Z");

    jobject key = env->GetObjectField(profile, keyField);
    jobject groups = nullptr;
    jobject capabilities = nullptr;
    jobject domain = nullptr;
    jobject templateName = nullptr;
    p_key_t p_key = {};
    int currentUid = 0;
    int uid = 0;
    int gid = 0;
    bool allowSu = false;
    bool umountModules = false;
    app_profile p = {};

    do {
        if (!key) {
            break;
        }
        if (!copyJStringToFixed(env, (jstring) key, p_key)) {
            break;
        }

        currentUid = env->GetIntField(profile, currentUidField);
        uid = env->GetIntField(profile, uidField);
        gid = env->GetIntField(profile, gidField);
        groups = env->GetObjectField(profile, groupsField);
        capabilities = env->GetObjectField(profile, capabilitiesField);
        domain = env->GetObjectField(profile, domainField);
        allowSu = env->GetBooleanField(profile, allowSuField);
        umountModules = env->GetBooleanField(profile, umountModulesField);

        p.version = KSU_APP_PROFILE_VER;
        strcpy(p.key, p_key);
        p.allow_su = allowSu;
        p.current_uid = currentUid;

        if (allowSu) {
            p.rp_config.use_default = env->GetBooleanField(profile, rootUseDefaultField);
            templateName = env->GetObjectField(profile, rootTemplateField);
            if (templateName && !copyJStringToFixed(env, (jstring) templateName, p.rp_config.template_name)) {
                break;
            }

            p.rp_config.profile.uid = uid;
            p.rp_config.profile.gid = gid;

            int groups_count = getListSize(env, groups);
            if (groups_count > KSU_MAX_GROUPS) {
                LOGD("groups count too large: %d", groups_count);
                break;
            }
            p.rp_config.profile.groups_count = groups_count;
            fillArrayWithList(env, groups, p.rp_config.profile.groups, groups_count);

            p.rp_config.profile.capabilities.effective = capListToBits(env, capabilities);

            if (!copyJStringToFixed(env, (jstring) domain, p.rp_config.profile.selinux_domain)) {
                break;
            }

            p.rp_config.profile.namespaces = env->GetIntField(profile, namespacesField);
        } else {
            p.nrp_config.use_default = env->GetBooleanField(profile, nonRootUseDefaultField);
            p.nrp_config.profile.umount_modules = umountModules;
        }

        success = set_app_profile(&p);
    } while (false);

    if (templateName) {
        env->DeleteLocalRef(templateName);
    }
    if (domain) {
        env->DeleteLocalRef(domain);
    }
    if (capabilities) {
        env->DeleteLocalRef(capabilities);
    }
    if (groups) {
        env->DeleteLocalRef(groups);
    }
    if (key) {
        env->DeleteLocalRef(key);
    }
    env->DeleteLocalRef(cls);
    return success;
}
extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_uidShouldUmount(JNIEnv *env, jobject thiz, jint uid) {
    return uid_should_umount(uid);
}
extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_isSuEnabled(JNIEnv *env, jobject thiz) {
    return is_su_enabled();
}
extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_setSuEnabled(JNIEnv *env, jobject thiz, jboolean enabled) {
    return set_su_enabled(enabled);
}

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_isKernelUmountEnabled(JNIEnv *env, jobject thiz) {
    return is_kernel_umount_enabled();
}

extern "C"
JNIEXPORT jboolean JNICALL
Java_com_qrj_apexsu_Natives_setKernelUmountEnabled(JNIEnv *env, jobject thiz, jboolean enabled) {
    return set_kernel_umount_enabled(enabled);
}

extern "C"
JNIEXPORT jstring JNICALL
Java_com_qrj_apexsu_Natives_getUserName(JNIEnv *env, jobject thiz, jint uid) {
    struct passwd *pw = getpwuid((uid_t) uid);
    if (pw && pw->pw_name && pw->pw_name[0] != '\0') {
        return env->NewStringUTF(pw->pw_name);
    }
    return nullptr;
}
