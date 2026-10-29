pub(crate) use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;
pub(crate) use worth_server::WorthServerDirectDeclarationDenialCode;

pub(crate) use super::certification_bundle::WorthServerCertificationOutputDigest as Output;
pub(crate) use super::certification_counter_assertions::{
    assert_counter_exact, assert_counters_zero,
};
pub(crate) use super::certification_worth_native_fixture::{
    cross_branch_lease_reuse_denial_bundle, cross_workspace_lease_reuse_denial_bundle,
    retained_artifact_denial_bundle, runtime_backed_missing_basis_denial_bundle,
    runtime_backed_stale_basis_denial_bundle, saved_query_intake_denial, standard_server,
};

#[test]
fn worth_native_sabotage_suite_blocks_direct_surface_shortcuts_and_leaves_zero_residue() {
    let compile_fail = trybuild::TestCases::new();
    compile_fail
        .compile_fail("tests/ui/construction/worth_native/private_prepared_session_constructor.rs");
    compile_fail.compile_fail("tests/ui/construction/worth_native/private_session_constructor.rs");
    compile_fail
        .compile_fail("tests/ui/construction/worth_native/private_direct_facade_constructor.rs");
    compile_fail
        .compile_fail("tests/ui/construction/worth_native/private_raw_query_workspace_access.rs");
    compile_fail.compile_fail(
        "tests/ui/construction/worth_native/private_direct_delivery_contract_constructor.rs",
    );
    compile_fail.compile_fail(
        "tests/ui/construction/worth_native/private_direct_lease_declaration_constructor.rs",
    );

    let server = standard_server();
    let retained_denial_lane = retained_artifact_denial_bundle(&server, "users.profile.missing");
    let missing_basis_lane = runtime_backed_missing_basis_denial_bundle(&server, "users.profile");
    let stale_basis_lane = runtime_backed_stale_basis_denial_bundle(&server, "users.profile");
    let cross_workspace_lane = cross_workspace_lease_reuse_denial_bundle(&server, "users.profile");
    let cross_branch_lane = cross_branch_lease_reuse_denial_bundle(&server, "users.profile");
    let saved_query_denial = saved_query_intake_denial();

    assert_eq!(
        retained_denial_lane.support_truth_kind(),
        FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
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
        missing_basis_lane.output_digest(Output::DenialCode),
        Some("RuntimeBackedResumeMissingBasis")
    );
    assert!(missing_basis_lane
        .output_digest(Output::DenialDetail)
        .expect("missing-basis denial detail")
        .contains(
            missing_basis_lane
                .output_digest(Output::Basis)
                .expect("missing-basis retained basis digest")
        ));

    assert_eq!(
        stale_basis_lane.output_digest(Output::DenialCode),
        Some("RuntimeBackedResumeStaleBasis")
    );
    let stale_basis_detail = stale_basis_lane
        .output_digest(Output::DenialDetail)
        .expect("stale-basis denial detail");
    assert!(stale_basis_detail.contains("basis:drifted"));
    assert!(stale_basis_detail.contains(
        stale_basis_lane
            .output_digest(Output::Basis)
            .expect("stale-basis retained basis digest")
    ));

    assert_eq!(
        cross_workspace_lane.output_digest(Output::DenialCode),
        Some("LeaseDeclarationContextMismatch")
    );
    assert!(cross_workspace_lane
        .output_digest(Output::DenialDetail)
        .expect("cross-workspace denial detail")
        .contains("workspace=`workspace-42`"));
    assert_ne!(
        cross_workspace_lane.output_digest(Output::Workspace),
        cross_branch_lane.output_digest(Output::Workspace)
    );

    assert_eq!(
        cross_branch_lane.output_digest(Output::DenialCode),
        Some("LeaseDeclarationContextMismatch")
    );
    let cross_branch_detail = cross_branch_lane
        .output_digest(Output::DenialDetail)
        .expect("cross-branch denial detail");
    assert!(cross_branch_detail.contains("branch=`main`"));
    assert!(cross_branch_detail.contains("branch=`branch:branch-9`"));

    assert_eq!(
        saved_query_denial.code(),
        WorthServerDirectDeclarationDenialCode::SourceNotAdmitted
    );
    assert_eq!(
        saved_query_denial
            .support_snapshot()
            .expect("saved-query denial should preserve support snapshot")
            .source_support_reason(),
        "saved-query declaration intake remains deferred until a later direct-consumption phase"
    );
}
