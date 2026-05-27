#[test]
fn patch_banner_and_lkm_log_use_apexsu_branding() {
    let banner = include_str!("../src/banner");
    let boot_patch = include_str!("../src/boot_patch.rs");

    assert!(banner.contains("ApexSU"));
    assert!(!banner.contains("KernelSU"));
    assert!(boot_patch.contains("- Adding ApexSU LKM"));
    assert!(!boot_patch.contains("- Adding KernelSU LKM"));
}
