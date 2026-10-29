//! Built product executables, colocated with the active test profile.
use std::sync::OnceLock;

fn executable(name: &str) -> String {
    let test = std::env::current_exe().expect("active integration executable");
    let profile = test.parent().and_then(std::path::Path::parent)
        .expect("Cargo integration executable is under profile/deps");
    let path = profile.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    assert!(path.is_file(), "product executable must be built by the court gate: {}", path.display());
    path.to_str().expect("Cargo executable path is UTF-8").to_owned()
}

pub fn physical_media_os_observer() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| executable("physical_media_os_observer")).as_str()
}

pub fn physical_store_integrity_observer() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| executable("physical_store_integrity_observer")).as_str()
}

pub fn physical_store_offline_observer() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| executable("physical_store_offline_observer")).as_str()
}

pub fn physical_store_recover() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| executable("physical_store_recover")).as_str()
}

pub fn store_test_runner() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| executable("store-test-runner")).as_str()
}
