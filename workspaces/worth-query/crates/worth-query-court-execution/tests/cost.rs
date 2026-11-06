//! Defined text symbols expose generic field-body fan-out in the actual probe archive.
use std::{path::PathBuf, process::Command};

#[test]
fn native_field_instantiation_stays_within_the_shared_core_ceiling() {
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
    let definitions: Vec<_> = text
        .lines()
        .filter(|line| matches!(line.split_whitespace().nth(1), Some("t" | "T")))
        .collect();
    let counts: Vec<_> = (0..40)
        .map(|aspect| {
            let marker = format!("::Fact<{aspect}>");
            definitions
                .iter()
                .filter(|line| line.contains(&marker))
                .count()
        })
        .collect();
    println!(
        "probe text symbols={}, per-aspect={counts:?}",
        definitions.len()
    );
    // Headroom admits ordinary lawful changes while refusing the measured field-body multiplier.
    assert!(
        definitions.len() <= 2_500,
        "field code generation exceeded its fixed ceiling"
    );
    assert!(
        counts.iter().all(|count| (1..=16).contains(count))
            && counts.windows(2).all(|pair| pair[0] == pair[1]),
        "additional aspects must retain a constant thin-shell cost"
    );
    assert!(
        !definitions
            .iter()
            .any(|line| line.contains("Application<") && line.contains("Fact<")),
        "field bodies must not be instantiated per application/member pair"
    );
}
