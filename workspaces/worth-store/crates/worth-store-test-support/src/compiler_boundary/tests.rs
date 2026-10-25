use std::path::{Path, PathBuf};

use super::{cargo_dependency_manifest, run_cargo_ui_fixture_suite, UiRunFailure};

#[test]
fn fixtures_in_one_environment_share_manifest_and_target_root() {
    let scratch = ScratchUiWorld::new("cache-sharing");
    let fixtures = [
        ("private_a.rs", ["private"].as_slice()),
        ("private_b.rs", ["private"].as_slice()),
    ];
    let evidence = run_cargo_ui_fixture_suite(
        scratch.root(),
        "cache-sharing",
        scratch.dependency_manifest(),
        "production",
        "diagnostic-test",
        scratch.fixture_root(),
        &fixtures,
    )
    .unwrap();

    assert_eq!(evidence.fixtures.len(), 2);
    assert!(evidence
        .shared_target_root
        .ends_with("store-ui/cargo-target"));
    assert!(evidence
        .fixtures
        .iter()
        .all(|fixture| fixture.fixture.environment_identity == evidence.environment_identity));
    assert!(evidence
        .fixtures
        .iter()
        .all(|fixture| fixture.semantic_denial_matched));
    let incremental = scratch
        .root()
        .join("target/store-ui/cargo-target/diagnostic-test/incremental");
    match std::fs::read_dir(incremental) {
        Ok(mut sessions) => assert!(
            sessions.next().is_none(),
            "repository configuration must not create incremental sessions for diagnostic proofs"
        ),
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::NotFound),
    }
}

#[test]
fn unrelated_compile_failure_cannot_satisfy_declared_semantic_denial() {
    let scratch = ScratchUiWorld::new("wrong-reason");
    let denial = run_cargo_ui_fixture_suite(
        scratch.root(),
        "wrong-reason",
        scratch.dependency_manifest(),
        "production",
        "diagnostic-test",
        scratch.fixture_root(),
        &[("unresolved.rs", &["private"])],
    )
    .unwrap_err();

    assert!(matches!(denial, UiRunFailure::WrongCompilerDenial { .. }));
    assert!(denial.to_string().contains("semantic fragment"));
}

#[test]
fn fixture_source_text_cannot_impersonate_a_semantic_compiler_message() {
    let expected = super::ExpectedCompilerDenial::semantic_fragments(["private"]).unwrap();
    let diagnostics = vec![super::CheckedCompilerDiagnostic {
        level: "error".to_owned(),
        code: Some("E0432".to_owned()),
        message: "unresolved import `absent_dependency`".to_owned(),
        rendered: "use absent_dependency::private;".to_owned(),
    }];

    let denial = super::diagnostics::validate_denial(&expected, &diagnostics, "").unwrap_err();

    assert!(denial.contains("missed semantic fragment"));
}

struct ScratchUiWorld {
    _directory: tempfile::TempDir,
    root: PathBuf,
    fixture_root: PathBuf,
}

impl ScratchUiWorld {
    fn new(label: &str) -> Self {
        let directory = tempfile::Builder::new()
            .prefix(&format!("worth-store-standard-ui-{label}-"))
            .tempdir()
            .unwrap();
        let root = directory.path().to_owned();
        let fixture_root = root.join("fixtures");
        std::fs::create_dir_all(root.join("dependency/src")).unwrap();
        std::fs::create_dir_all(&fixture_root).unwrap();
        std::fs::create_dir_all(root.join(".cargo")).unwrap();
        std::fs::write(
            root.join(".cargo/config.toml"),
            "[build]\nincremental = true\n",
        )
        .unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\n\n[workspace.package]\nedition = \"2021\"\nversion = \"0.0.0\"\nlicense = \"UNLICENSED\"\n\n[workspace.dependencies]\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n\n[profile.test]\n\n[profile.diagnostic-test]\ninherits = \"test\"\n",
        )
        .unwrap();
        std::fs::write(root.join("Cargo.lock"), "# UI harness identity lock\n").unwrap();
        std::fs::write(
            root.join("dependency/Cargo.toml"),
            "[package]\nname = \"sealed-dependency\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("dependency/src/lib.rs"),
            "pub struct Sealed { value: u64 }\n\
             impl Sealed { pub fn value(&self) -> u64 { self.value } }\n",
        )
        .unwrap();
        for fixture in ["private_a.rs", "private_b.rs"] {
            std::fs::write(
                fixture_root.join(fixture),
                "use sealed_dependency::Sealed; fn main() { let _ = Sealed { value: 1 }; }\n",
            )
            .unwrap();
        }
        std::fs::write(
            fixture_root.join("unresolved.rs"),
            "use absent_dependency::Missing; fn main() {}\n",
        )
        .unwrap();
        Self {
            _directory: directory,
            root,
            fixture_root,
        }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn fixture_root(&self) -> &Path {
        &self.fixture_root
    }

    fn dependency_manifest(&self) -> String {
        cargo_dependency_manifest(
            &[(
                "sealed-dependency",
                self.root.join("dependency").as_path(),
                &[],
            )],
            &[],
        )
    }
}
