//! Defined text symbols expose generic field-body fan-out in the actual probe archive.
use std::{path::PathBuf, process::Command};
mod budget;

#[test]
fn native_field_instantiation_stays_within_the_shared_core_ceiling() {
    let definitions = probe_symbols();
    budget::assert_kind(&definitions, budget::OWNED_ASPECT);
    budget::assert_total(&definitions);
}

#[test]
fn extension_instantiation_stays_within_the_shared_core_ceiling() {
    let definitions = probe_symbols();
    budget::assert_kind(&definitions, budget::EXTENSION_ASPECT);
    budget::assert_total(&definitions);
}

fn probe_symbols() -> Vec<String> {
    std::hint::black_box(worth_query_testkit_instantiation::extension_relations);
    std::hint::black_box(worth_query_testkit_instantiation::extension_views);
    std::hint::black_box(worth_query_testkit_instantiation::extension_application_0);
    std::hint::black_box(worth_query_testkit_instantiation::extension_application_1);
    std::hint::black_box(worth_query_testkit_instantiation::extension_application_2);
    std::hint::black_box(worth_query_testkit_instantiation::extension_application_3);
    std::hint::black_box(worth_query_testkit_instantiation::application_0);
    std::hint::black_box(worth_query_testkit_instantiation::application_1);
    std::hint::black_box(worth_query_testkit_instantiation::application_2);
    std::hint::black_box(worth_query_testkit_instantiation::application_3);
    let directory = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let mut archives: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libworth_query_testkit_instantiation-")
                && path.extension().is_some_and(|ext| ext == "rlib")
        })
        .collect();
    archives.sort_by_key(|path| std::fs::metadata(path).unwrap().modified().unwrap());
    let archive = archives.last().expect("cargo linked the probe archive");
    let sysroot = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .unwrap();
    assert!(sysroot.status.success());
    let version = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(version.status.success());
    let version = String::from_utf8(version.stdout).unwrap();
    let host = version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap();
    let tool = PathBuf::from(String::from_utf8(sysroot.stdout).unwrap().trim())
        .join("lib/rustlib")
        .join(host)
        .join("bin")
        .join(if cfg!(windows) {
            "llvm-nm.exe"
        } else {
            "llvm-nm"
        });
    let result = Command::new(tool)
        .args(["--defined-only", "--demangle"])
        .arg(archive)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "llvm-nm failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8(result.stdout).unwrap();
    text.lines()
        .filter(|line| matches!(line.split_whitespace().nth(1), Some("t" | "T")))
        .map(str::to_owned)
        .collect()
}

#[test]
fn extension_relation_instantiation_stays_within_the_shared_core_ceiling() {
    let definitions = probe_symbols();
    budget::assert_kind(&definitions, budget::RELATION);
    budget::assert_kind(&definitions, budget::PAYLOAD_ASPECT);
    budget::assert_total(&definitions);
}
