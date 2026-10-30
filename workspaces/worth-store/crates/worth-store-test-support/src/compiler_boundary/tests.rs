use std::path::{Path, PathBuf};

use super::{read_ui_fixture_suite, UiCompilerEnvironment, UiRunFailure};

#[test]
fn fixtures_in_one_environment_share_manifest_and_target_root() {
    let root = compiler_unit_root();
    let workspace = store_workspace_root();
    let evidence = read_ui_fixture_suite(
        &workspace,
        "cache-sharing",
        UiCompilerEnvironment::Production,
        &root.join("fixtures"),
        &[
            ("private_a.rs", &["private"]),
            ("private_b.rs", &["private"]),
        ],
    )
    .unwrap();

    assert_eq!(evidence.fixtures.len(), 2);
    assert!(!evidence.environment_identity.is_empty());
    let target = active_target_root(&workspace);
    let observed = Path::new(&evidence.shared_target_root);
    let observed = if observed.is_absolute() {
        observed.to_path_buf()
    } else {
        workspace.join(observed)
    };
    assert_eq!(observed.canonicalize().unwrap(), target);
    assert!(evidence
        .fixtures
        .iter()
        .all(|fixture| fixture.fixture.environment_identity == evidence.environment_identity));
    assert!(evidence
        .fixtures
        .iter()
        .all(|fixture| fixture.semantic_denial_matched));
    let incremental = target.join("diagnostic-test/incremental");
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
    let root = compiler_unit_root();
    let denial = read_ui_fixture_suite(
        &store_workspace_root(),
        "wrong-reason",
        UiCompilerEnvironment::Production,
        &root.join("fixtures"),
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

fn compiler_unit_root() -> PathBuf {
    std::env::var_os("WORTH_PREBUILT_COMPILER_UNIT_ROOT")
        .map(PathBuf::from)
        .expect("workspace prebuild must compile the real compiler-unit fixtures")
}

fn store_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn active_target_root(workspace: &Path) -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target"));
    let target = if target.is_absolute() {
        target
    } else {
        workspace.join(target)
    };
    target
        .canonicalize()
        .expect("active compiler target exists")
}
