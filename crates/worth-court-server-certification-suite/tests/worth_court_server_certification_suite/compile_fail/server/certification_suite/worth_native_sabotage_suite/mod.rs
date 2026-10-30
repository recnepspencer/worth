pub(crate) use {crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counter_exact, crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counters_zero};
pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::cross_branch_lease_reuse_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::cross_workspace_lease_reuse_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::retained_artifact_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::runtime_backed_missing_basis_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::runtime_backed_stale_basis_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::saved_query_intake_denial, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::standard_server};

#[test]
fn worth_native_sabotage_suite_blocks_direct_surface_shortcuts_and_leaves_zero_residue() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/worth_native/private_prepared_session_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/worth_native/private_session_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/worth_native/private_direct_facade_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/worth_native/private_raw_query_workspace_access.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/worth_native/private_direct_delivery_contract_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/worth_native/private_direct_lease_declaration_constructor.rs")).expect("prebuilt compiler denial");

    let server = standard_server();
    let retained_denial_lane = retained_artifact_denial_bundle(&server, "users.profile.missing");
    let missing_basis_lane = runtime_backed_missing_basis_denial_bundle(&server, "users.profile");
    let stale_basis_lane = runtime_backed_stale_basis_denial_bundle(&server, "users.profile");
    let cross_workspace_lane = cross_workspace_lease_reuse_denial_bundle(&server, "users.profile");
    let cross_branch_lane = cross_branch_lease_reuse_denial_bundle(&server, "users.profile");
    let saved_query_denial = saved_query_intake_denial();

    assert_eq!(
        retained_denial_lane.support_truth_kind(),
        worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
    );
    assert_counter_exact(
        &retained_denial_lane,
        "response.query_handoff_denial.count",
        1,
    );
    assert_counters_zero(
        &retained_denial_lane,
        &[
            "response.success.count",
            "response.query_read_success.count",
            "response.query_mutation_success.count",
            "response.downstream_delivery_success.count",
        ],
    );

    for lane in [
        &missing_basis_lane,
        &stale_basis_lane,
        &cross_workspace_lane,
        &cross_branch_lane,
    ] {
        assert_counter_exact(lane, "response.query_handoff_denial.count", 1);
        assert_counters_zero(
            lane,
            &[
                "response.success.count",
                "response.query_read_success.count",
                "response.downstream_delivery_success.count",
            ],
        );
    }

    assert_eq!(
        missing_basis_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
        Some("RuntimeBackedResumeMissingBasis")
    );
    assert!(missing_basis_lane
        .output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialDetail)
        .expect("missing-basis denial detail")
        .contains(
            missing_basis_lane
                .output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis)
                .expect("missing-basis retained basis digest")
        ));

    assert_eq!(
        stale_basis_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
        Some("RuntimeBackedResumeStaleBasis")
    );
    let stale_basis_detail = stale_basis_lane
        .output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialDetail)
        .expect("stale-basis denial detail");
    assert!(stale_basis_detail.contains("basis:drifted"));
    assert!(stale_basis_detail.contains(
        stale_basis_lane
            .output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis)
            .expect("stale-basis retained basis digest")
    ));

    assert_eq!(
        cross_workspace_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
        Some("LeaseDeclarationContextMismatch")
    );
    assert!(cross_workspace_lane
        .output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialDetail)
        .expect("cross-workspace denial detail")
        .contains("workspace=`workspace-42`"));
    assert_ne!(
        cross_workspace_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
        cross_branch_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace)
    );

    assert_eq!(
        cross_branch_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
        Some("LeaseDeclarationContextMismatch")
    );
    let cross_branch_detail = cross_branch_lane
        .output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialDetail)
        .expect("cross-branch denial detail");
    assert!(cross_branch_detail.contains("branch=`main`"));
    assert!(cross_branch_detail.contains("branch=`branch:branch-9`"));

    assert_eq!(
        saved_query_denial.code(),
        worth_server::WorthServerDirectDeclarationDenialCode::SourceNotAdmitted
    );
    assert_eq!(
        saved_query_denial
            .support_snapshot()
            .expect("saved-query denial should preserve support snapshot")
            .source_support_reason(),
        "saved-query declaration intake remains deferred until a later direct-consumption phase"
    );
}
