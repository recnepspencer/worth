use super::*;
use std::process::Command;

fn directory(name: &str) -> PathBuf {
    let base =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/m271-editions/content-courts");
    let path = base.join(format!(
        "{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    fs::canonicalize(path).unwrap()
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[test]
fn locked_git_tree_checks_actual_bytes_and_excludes_untracked_compiler_inputs() {
    let root = directory("git");
    fs::write(root.join("lib.rs"), "pub const VALUE:u8=17;\n").unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "lib.rs"]);
    git(
        &root,
        &[
            "-c",
            "user.name=Edition fixture",
            "-c",
            "user.email=edition@example.invalid",
            "commit",
            "-q",
            "-m",
            "locked source",
        ],
    );
    let revision = git(&root, &["rev-parse", "HEAD"]);
    let package = json!({"root":root,"name":"locked-example","version":"0.1.0","source":format!("git+file:///fixture#{revision}")});
    let session = json!({"packages":[package.clone()]});
    let verified = verify(&package, &session).unwrap();
    assert_eq!(
        verified.files[Path::new("lib.rs")],
        b"pub const VALUE:u8=17;\n"
    );
    fs::write(root.join("untracked.rs"), "pub const EXTRA:u8=1;").unwrap();
    assert!(!contains_input(
        &verified,
        &root,
        &root.join("untracked.rs")
    ));
    fs::write(root.join("lib.rs"), "pub const VALUE:u8=23;\n").unwrap();
    assert!(verify(&package, &session)
        .err()
        .unwrap()
        .contains("differs from locked archive/tree"));
    git(&root, &["add", "lib.rs"]);
    git(
        &root,
        &[
            "-c",
            "user.name=Edition fixture",
            "-c",
            "user.email=edition@example.invalid",
            "commit",
            "-q",
            "-m",
            "substituted source",
        ],
    );
    let substituted = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["replace", &revision, &substituted]);
    assert_eq!(
        git(&root, &["show", &format!("{revision}:lib.rs")]),
        "pub const VALUE:u8=23;"
    );
    assert!(verify(&package, &session)
        .err()
        .unwrap()
        .contains("differs from locked archive/tree"));
    fs::write(root.join("lib.rs"), "pub const VALUE:u8=17;\n").unwrap();
    assert!(
        verify(&package, &session).is_ok(),
        "locked A remains authoritative while its replacement ref exists"
    );
}

pub(crate) fn serde_package() -> Value {
    let lock: toml::Value = toml::from_str(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock")).unwrap(),
    )
    .unwrap();
    let package = lock["package"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| {
            p["name"].as_str() == Some("serde_core") && p["version"].as_str() == Some("1.0.229")
        })
        .unwrap();
    json!({"name":"serde_core","version":"1.0.229","checksum":package["checksum"].as_str().unwrap(),"source":package["source"].as_str().unwrap()})
}

#[test]
fn registry_copy_cannot_launder_tampering_or_a_wrong_archive_checksum() {
    let root = directory("registry");
    let mut package = serde_package();
    let files = registry::files(&package).unwrap();
    for (path, content) in &files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    package["root"] = json!(root);
    let session = json!({"packages":[package.clone()]});
    assert_eq!(verify(&package, &session).unwrap().files.len(), files.len());
    fs::write(root.join("src/lib.rs"), b"// tampered\n").unwrap();
    fs::write(
        root.join(".cargo-checksum.json"),
        b"{\"files\":{},\"package\":null}",
    )
    .unwrap();
    assert!(verify(&package, &session)
        .err()
        .unwrap()
        .contains("differs from locked archive/tree"));
    package["checksum"] = json!("0".repeat(64));
    assert!(verify(&package, &session)
        .err()
        .unwrap()
        .contains("matches the locked checksum"));
}

pub(crate) fn serde_generator() -> String {
    String::from_utf8(
        registry::files(&serde_package())
            .unwrap()
            .remove(Path::new("build.rs"))
            .unwrap(),
    )
    .unwrap()
}
