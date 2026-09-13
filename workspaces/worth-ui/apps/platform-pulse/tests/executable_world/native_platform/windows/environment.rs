pub(super) fn qualified_windows_11_version(version: &str) -> bool {
    version
        .strip_prefix("Microsoft Windows [Version 10.0.")
        .and_then(|tail| tail.strip_suffix(']'))
        .and_then(|tail| tail.split('.').next())
        .and_then(|build| build.parse::<u32>().ok())
        .is_some_and(|build| build >= 22_000)
}

#[test]
fn windows_11_build_classifier_rejects_windows_10_and_malformed_versions() {
    assert!(qualified_windows_11_version(
        "Microsoft Windows [Version 10.0.26100.4652]"
    ));
    assert!(!qualified_windows_11_version(
        "Microsoft Windows [Version 10.0.19045.4652]"
    ));
    assert!(!qualified_windows_11_version("Microsoft Windows 10"));
}
