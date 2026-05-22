use std::mem::{align_of, offset_of, size_of};

use apexsu_kernel_abi_checker::{
    AppProfile, KSU_APP_PROFILE_VER, KSU_MAX_GROUPS, KSU_MAX_PACKAGE_NAME, KSU_SELINUX_DOMAIN,
    KsuAddTryUmountCmd, KsuCompatSepolData, KsuGetAllowListCmd, KsuGetAppProfileCmd,
    KsuGetFeatureCmd, KsuGetInfoCmd, KsuManageMarkCmd, KsuNewGetAllowListCmd, KsuSepolData,
    KsuSetAppProfileCmd, KsuSetFeatureCmd, KsuSetSepolicyCmd, NonRootProfile,
    ProfileValidationError, RootProfile, validate_app_profile,
};

#[test]
fn app_profile_constants_match_kernel_headers() {
    assert_eq!(KSU_APP_PROFILE_VER, 2);
    assert_eq!(KSU_MAX_PACKAGE_NAME, 256);
    assert_eq!(KSU_MAX_GROUPS, 32);
    assert_eq!(KSU_SELINUX_DOMAIN, 64);
}

#[test]
fn app_profile_layout_matches_c_abi() {
    assert_eq!(size_of::<RootProfile>(), 240);
    assert_eq!(align_of::<RootProfile>(), 8);
    assert_eq!(size_of::<NonRootProfile>(), 1);
    assert_eq!(align_of::<NonRootProfile>(), 1);
    assert_eq!(size_of::<AppProfile>(), 776);
    assert_eq!(align_of::<AppProfile>(), 8);
    assert_eq!(size_of::<KsuGetAppProfileCmd>(), 776);
    assert_eq!(size_of::<KsuSetAppProfileCmd>(), 776);
}

#[test]
fn supercall_command_layouts_match_c_abi() {
    assert_eq!(size_of::<KsuGetAllowListCmd>(), 520);
    assert_eq!(align_of::<KsuGetAllowListCmd>(), 4);
    assert_eq!(size_of::<KsuGetInfoCmd>(), 12);
    assert_eq!(align_of::<KsuGetInfoCmd>(), 4);
    assert_eq!(offset_of!(KsuGetInfoCmd, version), 0);
    assert_eq!(offset_of!(KsuGetInfoCmd, flags), 4);
    assert_eq!(offset_of!(KsuGetInfoCmd, features), 8);
    assert_eq!(size_of::<KsuNewGetAllowListCmd>(), 4);
    assert_eq!(align_of::<KsuNewGetAllowListCmd>(), 4);
    assert_eq!(size_of::<KsuGetFeatureCmd>(), 24);
    assert_eq!(align_of::<KsuGetFeatureCmd>(), 8);
    assert_eq!(size_of::<KsuSetFeatureCmd>(), 16);
    assert_eq!(size_of::<KsuManageMarkCmd>(), 12);
    assert_eq!(size_of::<KsuAddTryUmountCmd>(), 16);
}

#[test]
fn sepolicy_payload_layouts_match_c_abi() {
    assert_eq!(size_of::<KsuSetSepolicyCmd>(), 16);
    assert_eq!(align_of::<KsuSetSepolicyCmd>(), 8);
    assert_eq!(offset_of!(KsuSetSepolicyCmd, cmd), 0);
    assert_eq!(offset_of!(KsuSetSepolicyCmd, arg), 8);

    assert_eq!(size_of::<KsuSepolData>(), 64);
    assert_eq!(align_of::<KsuSepolData>(), 8);
    assert_eq!(offset_of!(KsuSepolData, cmd), 0);
    assert_eq!(offset_of!(KsuSepolData, subcmd), 4);
    assert_eq!(offset_of!(KsuSepolData, sepol1), 8);
    assert_eq!(offset_of!(KsuSepolData, sepol7), 56);

    assert_eq!(size_of::<KsuCompatSepolData>(), 36);
    assert_eq!(align_of::<KsuCompatSepolData>(), 4);
    assert_eq!(offset_of!(KsuCompatSepolData, cmd), 0);
    assert_eq!(offset_of!(KsuCompatSepolData, subcmd), 4);
    assert_eq!(offset_of!(KsuCompatSepolData, sepol1), 8);
    assert_eq!(offset_of!(KsuCompatSepolData, sepol7), 32);
}

#[test]
fn app_profile_validator_accepts_valid_root_profile() {
    let mut profile = AppProfile::new_for_tests("com.example.app", 10_000);
    profile.allow_su = true;
    profile.root_config_mut_for_tests().profile.groups_count = KSU_MAX_GROUPS as i32;
    profile
        .root_config_mut_for_tests()
        .profile
        .set_selinux_domain_for_tests("u:r:su_app:s0");

    assert_eq!(validate_app_profile(&profile), Ok(()));
}

#[test]
fn app_profile_validator_rejects_old_version() {
    let mut profile = AppProfile::new_for_tests("com.example.app", 10_000);
    profile.version = KSU_APP_PROFILE_VER - 1;

    assert_eq!(
        validate_app_profile(&profile),
        Err(ProfileValidationError::UnsupportedVersion)
    );
}

#[test]
fn app_profile_validator_rejects_missing_key_terminator() {
    let profile = AppProfile {
        key: [b'a'; KSU_MAX_PACKAGE_NAME],
        ..AppProfile::default()
    };

    assert_eq!(
        validate_app_profile(&profile),
        Err(ProfileValidationError::InvalidKey)
    );
}

#[test]
fn app_profile_validator_rejects_negative_group_count() {
    let mut profile = AppProfile::new_for_tests("com.example.app", 10_000);
    profile.allow_su = true;
    profile.root_config_mut_for_tests().profile.groups_count = -1;
    profile
        .root_config_mut_for_tests()
        .profile
        .set_selinux_domain_for_tests("u:r:su_app:s0");

    assert_eq!(
        validate_app_profile(&profile),
        Err(ProfileValidationError::InvalidGroupsCount)
    );
}

#[test]
fn app_profile_validator_rejects_missing_selinux_domain_terminator() {
    let mut profile = AppProfile::new_for_tests("com.example.app", 10_000);
    profile.allow_su = true;
    profile.root_config_mut_for_tests().profile.selinux_domain = [b'a'; KSU_SELINUX_DOMAIN];

    assert_eq!(
        validate_app_profile(&profile),
        Err(ProfileValidationError::InvalidSelinuxDomain)
    );
}

#[test]
fn app_profile_validator_checks_default_root_profile_by_special_key() {
    let mut profile = AppProfile::new_for_tests("#", 0);
    profile.allow_su = false;
    profile.root_config_mut_for_tests().profile.groups_count = KSU_MAX_GROUPS as i32 + 1;
    profile
        .root_config_mut_for_tests()
        .profile
        .set_selinux_domain_for_tests("u:r:su_app:s0");

    assert_eq!(
        validate_app_profile(&profile),
        Err(ProfileValidationError::InvalidGroupsCount)
    );
}

#[test]
fn app_profile_validator_allows_empty_template_name() {
    let mut profile = AppProfile::new_for_tests("com.example.app", 10_000);
    profile.allow_su = true;
    profile.root_config_mut_for_tests().template_name = [0; KSU_MAX_PACKAGE_NAME];
    profile
        .root_config_mut_for_tests()
        .profile
        .set_selinux_domain_for_tests("u:r:su_app:s0");

    assert_eq!(validate_app_profile(&profile), Ok(()));
}
