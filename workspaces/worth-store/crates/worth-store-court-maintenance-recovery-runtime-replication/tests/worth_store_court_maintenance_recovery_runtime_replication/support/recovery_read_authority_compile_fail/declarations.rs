pub(crate) use std::path::Path;

pub(crate) use {
    worth_store_test_support::compiler_boundary::read_ui_proof_suite,
    worth_store_test_support::compiler_boundary::ExpectedCompilerDenial,
    worth_store_test_support::compiler_boundary::UiCompilerEnvironment,
    worth_store_test_support::compiler_boundary::UiFixtureDeclaration,
    worth_store_test_support::compiler_boundary::UiProofSuiteDeclaration,
};

pub(crate) fn suite(root: &Path) -> UiProofSuiteDeclaration {
    let source_root = root.join(
        "crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/compile_fail/physical_isolation/recovery_read_authority",
    );
    let fixtures = fixture_denials()
        .into_iter()
        .map(|(name, fragments)| {
            worth_store_test_support::compiler_boundary::UiFixtureDeclaration::new(
                name.trim_end_matches(".rs"),
                source_root.join(name),
                worth_store_test_support::compiler_boundary::ExpectedCompilerDenial::semantic_fragments(fragments).unwrap(),
            )
            .unwrap()
        })
        .collect();
    worth_store_test_support::compiler_boundary::UiProofSuiteDeclaration::new(
        "recovery-read-authority",
        worth_store_test_support::compiler_boundary::UiCompilerEnvironment::Certification,
        fixtures,
    )
    .unwrap()
}

pub(crate) fn fixture_denials() -> Vec<(&'static str, Vec<&'static str>)> {
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

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("certification crate lives under the Store workspace")
}

pub(crate) fn manifest_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
