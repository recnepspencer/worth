use std::path::Path;

use worth_store_test_support::compiler_boundary::{
    run_ui_proof_suite, ExpectedCompilerDenial, UiFixtureDeclaration, UiProofEnvironment,
    UiProofSuiteDeclaration,
};

#[test]
fn recovery_reports_cannot_reenter_physical_read_authority() {
    let root = store_workspace_root();
    let evidence = run_ui_proof_suite(root, &suite(root)).unwrap();
    assert_eq!(evidence.fixtures.len(), fixture_denials().len());
    assert!(evidence
        .fixtures
        .iter()
        .all(|fixture| fixture.semantic_denial_matched));
}

fn suite(root: &Path) -> UiProofSuiteDeclaration {
    let source_root = root.join(
        "crates/worth-store-certification/tests/compile_fail/physical_isolation/recovery_read_authority",
    );
    let fixtures = fixture_denials()
        .into_iter()
        .map(|(name, fragments)| {
            UiFixtureDeclaration::new(
                name.trim_end_matches(".rs"),
                source_root.join(name),
                ExpectedCompilerDenial::semantic_fragments(fragments).unwrap(),
            )
            .unwrap()
        })
        .collect();
    UiProofSuiteDeclaration::new(
        "recovery-read-authority",
        UiProofEnvironment::cargo(dependency_manifest(root), "production", "diagnostic-test")
            .unwrap(),
        fixtures,
    )
    .unwrap()
}

fn fixture_denials() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "removed_publication_execution_exports.rs",
            vec![
                "unresolved import",
                "PhysicalPublicationReceipt",
                "AtomicPhysicalRootSwap",
                "ExecutedPublicationRecoveryReceipt",
            ],
        ),
        (
            "local_publication_cannot_be_completed_root.rs",
            vec![
                "mismatched types",
                "PhysicalPublicationPlanCompletion",
                "CompletedPhysicalRootPublication",
            ],
        ),
        (
            "local_publication_cannot_issue_executed_evidence.rs",
            vec![
                "no method named",
                "lower_to_foundational_evidence",
                "PhysicalPublicationPlanCompletion",
            ],
        ),
        (
            "removed_compaction_recovery_evidence.rs",
            vec!["unresolved import", "CompactionRecoveryEvidence"],
        ),
        (
            "local_compaction_completion_cannot_be_byte_receipt.rs",
            vec![
                "mismatched types",
                "CompactionReadPlanCompletion",
                "StablePhysicalReadReceipt",
            ],
        ),
        (
            "removed_entry_identity.rs",
            vec!["unresolved import", "PhysicalIsolationEntryIdentity"],
        ),
        (
            "removed_entry_admission.rs",
            vec!["unresolved import", "admit_physical_isolation_entry"],
        ),
        (
            "recovery_completion_cannot_be_root_basis.rs",
            vec![
                "From<RecoveryCompletion>",
                "PhysicalIsolationRootEpochBasis",
            ],
        ),
        (
            "recovery_completion_cannot_capture_reader.rs",
            vec![
                "mismatched types",
                "ServingPhysicalRuntime",
                "RecoveryCompletion",
            ],
        ),
    ]
}

fn dependency_manifest(root: &Path) -> String {
    format!(
        "[dependencies]\nworth-store-physical-certification = {{ path = \"{}\" }}\nworth-store-physical-isolation = {{ path = \"{}\" }}\nworth-store = {{ path = \"{}\" }}\nworth-store-recovery-runtime = {{ path = \"{}\" }}\n",
        manifest_path(&root.join("crates/worth-store-physical-certification")),
        manifest_path(&root.join("crates/worth-store-physical-isolation")),
        manifest_path(&root.join("crates/worth-store")),
        manifest_path(&root.join("crates/worth-store-recovery-runtime")),
    )
}

fn store_workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("certification crate lives under the Store workspace")
}

fn manifest_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
