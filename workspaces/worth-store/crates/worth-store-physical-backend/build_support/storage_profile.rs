use std::path::Path;

pub const PROFILE_INPUTS: &[&str] = &[
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_CFG_RACY_ASSERTS",
    "CARGO_CFG_TARGET_OS",
    "CARGO_CFG_TARGET_POINTER_WIDTH",
    "TARGET",
];

/// Qualifies compiler inputs and resolved package metadata for storage mechanics,
/// not media authority. The supported composition additionally assumes the
/// audited pinned dependency sources are unmodified. Cargo.lock is not source
/// byte attestation: registry replacement/vendor substitution can preserve its
/// registry labels and is outside this supported composition.
pub fn is_supported(compiler: &str, inputs: &[(&str, String)], lock: &str) -> bool {
    let value = |name| {
        inputs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
            .unwrap_or("")
    };
    compiler == "rustc 1.94.0 (4a4ef493e 2026-03-02)"
        && value("TARGET") == "x86_64-pc-windows-msvc"
        && value("CARGO_CFG_TARGET_OS") == "windows"
        && value("CARGO_CFG_TARGET_POINTER_WIDTH") == "64"
        && approved_flags(value("CARGO_ENCODED_RUSTFLAGS"))
        && PROFILE_INPUTS[1..4]
            .iter()
            .all(|name| value(name).is_empty())
        && [
            ("cap-std", "4.0.2"),
            ("cap-fs-ext", "4.0.2"),
            ("cap-primitives", "4.0.2"),
            ("maybe-owned", "0.3.4"),
        ]
        .iter()
        .all(|(name, version)| pinned_registry_package(lock, name, version))
}

fn approved_flags(encoded: &str) -> bool {
    if encoded.is_empty() {
        return true;
    }
    let mut flags = encoded.split('\u{1f}');
    while let Some(flag) = flags.next() {
        let option = if flag == "-C" {
            flags.next().map(|value| ("codegen", value))
        } else if flag == "--cfg" {
            flags.next().map(|value| ("cfg", value))
        } else if let Some(value) = flag.strip_prefix("-C") {
            Some(("codegen", value))
        } else {
            flag.strip_prefix("--cfg=").map(|value| ("cfg", value))
        };
        match option {
            Some(("codegen", "overflow-checks=on"))
            | Some(("cfg", "worth_ui_windowing=\"wayland\""))
            | Some(("cfg", "worth_ui_adapter=\"hardware\"")) => {}
            _ => return false,
        }
    }
    true
}

fn pinned_registry_package(lock: &str, expected_name: &str, expected_version: &str) -> bool {
    let mut matches = 0;
    for package in lock.split("[[package]]").skip(1) {
        let field = |key: &str| {
            package.lines().find_map(|line| {
                line.trim()
                    .strip_prefix(key)?
                    .strip_prefix(" = \"")?
                    .strip_suffix('"')
            })
        };
        if field("name") == Some(expected_name) && field("version") == Some(expected_version) {
            matches += 1;
            if field("source") != Some("registry+https://github.com/rust-lang/crates.io-index")
                || !field("checksum").is_some_and(|checksum| {
                    checksum.len() == 64 && checksum.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            {
                return false;
            }
        }
    }
    matches == 1
}

pub fn read_lock(path: &Path) -> String {
    // Bound the metadata read. The package predicate rejects missing/ambiguous
    // expected entries or unsupported source labels, but this metadata cannot
    // detect modified sources or registry replacement/vendor substitution.
    let Ok(metadata) = std::fs::metadata(path) else {
        return String::new();
    };
    if metadata.len() > 4 * 1024 * 1024 {
        return String::new();
    }
    std::fs::read_to_string(path).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock() -> String {
        [("cap-std", "4.0.2"), ("cap-fs-ext", "4.0.2"), ("cap-primitives", "4.0.2"), ("maybe-owned", "0.3.4")]
            .into_iter().map(|(name, version)| format!("[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{}\"\n", "a".repeat(64))).collect()
    }

    #[test]
    fn lock_metadata_read_is_bounded_and_missing_input_does_not_qualify() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Cargo.lock");
        assert!(read_lock(&path).is_empty());
        let expected = lock();
        std::fs::write(&path, &expected).unwrap();
        assert_eq!(read_lock(&path), expected);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(4 * 1024 * 1024 + 1)
            .unwrap();
        assert!(read_lock(&path).is_empty());
    }

    #[test]
    fn only_uninjected_pinned_registry_profile_is_supported() {
        const COMPILER: &str = "rustc 1.94.0 (4a4ef493e 2026-03-02)";
        let mut inputs = vec![
            ("CARGO_CFG_TARGET_OS", "windows".into()),
            ("CARGO_CFG_TARGET_POINTER_WIDTH", "64".into()),
            ("TARGET", "x86_64-pc-windows-msvc".into()),
        ];
        let lock = lock();
        assert!(is_supported(COMPILER, &inputs, &lock));
        for variable in &PROFILE_INPUTS[..4] {
            inputs.push((variable, "injected".into()));
            assert!(!is_supported(COMPILER, &inputs, &lock));
            inputs.pop();
        }
        for profile in ["-C\u{1f}overflow-checks=on\u{1f}--cfg\u{1f}worth_ui_windowing=\"wayland\"\u{1f}--cfg\u{1f}worth_ui_adapter=\"hardware\"", "-Coverflow-checks=on\u{1f}--cfg=worth_ui_windowing=\"wayland\"\u{1f}--cfg=worth_ui_adapter=\"hardware\""] {
            inputs.push(("CARGO_ENCODED_RUSTFLAGS", profile.into()));
            assert!(is_supported(COMPILER, &inputs, &lock));
            inputs.pop();
        }
        for profile in [
            "--cfg\u{1f}racy_asserts",
            "--sysroot=other",
            "-Zrandomize-layout",
            "--cfg",
            "-C",
            "-C\u{1f}\u{1f}overflow-checks=on",
            "--cfg=worth_ui_adapter=\"software\"",
        ] {
            assert!(!approved_flags(profile));
        }
        assert!(!is_supported("rustc 1.95.0 (other)", &inputs, &lock));
        assert!(!is_supported(
            "rustc 1.94.0 (different 2026-03-02)",
            &inputs,
            &lock
        ));
        assert!(!is_supported(
            COMPILER,
            &inputs,
            &lock.replace(
                "registry+https://github.com/rust-lang/crates.io-index",
                "git+https://example.invalid/replacement"
            )
        ));
        assert!(!is_supported(COMPILER, &inputs, &(lock.clone() + &lock)));
        assert!(!is_supported(
            COMPILER,
            &inputs,
            &lock.replace("version = \"4.0.2\"", "version = \"4.0.3\"")
        ));
        assert!(is_supported(
            COMPILER,
            &inputs,
            &(lock.clone()
                + &lock
                    .replace("version = \"4.0.2\"", "version = \"4.0.3\"")
                    .replace("version = \"0.3.4\"", "version = \"0.3.5\""))
        ));
        inputs
            .iter_mut()
            .find(|(key, _)| *key == "TARGET")
            .unwrap()
            .1 = "aarch64-pc-windows-msvc".into();
        assert!(!is_supported(COMPILER, &inputs, &lock));
    }
}
