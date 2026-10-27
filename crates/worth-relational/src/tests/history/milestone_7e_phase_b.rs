use crate::facade::history::BranchId;
use crate::facade::merge::MergeIntent;
use crate::facade::runtime::RelationalRuntime;
use crate::history::data::{
    RelationalMergeBranchBasis, RelationalMergeBranchBasisFoundationalLoweringDenial,
};
use crate::tests::support::{
    checkpoint_and_recover_with, create_branch_from_main, create_entity,
    create_entity_outcome_on_branch, durable_runtime_with_test_schema,
};
use crate::transactions::data::PublishedMergeExecutionAuthority;
use worth_foundational::{
    foundational_boundary_current_basis_readmission_authority, request_foundational_profile_set,
    AdmissionReadinessProfile, CanonicalBasisDomain, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    ExecutionObjectiveProfile, FoundationalBoundaryArtifactCategory,
    FoundationalBoundaryArtifactRole, FoundationalBoundaryCurrentBasisAuthority,
    FoundationalBoundaryCurrentBasisCertified, FoundationalBoundaryMaterializationSeam,
    FoundationalBoundaryMaterializationSource, FoundationalProfileSet, FoundationalProfileSetInput,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::Proof;
use worth_proof::TransitionOutcome;

#[test]
fn foundational_current_basis_lowering_preserves_exact_relational_basis_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let basis = runtime
        .history()
        .resolve_merge_branch_basis(
            &BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("relational basis");

    let lowered = runtime
        .history()
        .lower_merge_branch_basis_to_foundational_current_basis(
            &basis,
            version("m7e.phase2.exact"),
            materialized_profile(),
        )
        .expect("foundational current basis");

    assert_exact_branch_basis(lowered.basis(), &basis);
    assert_eq!(
        lowered.strong_basis().payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    assert_eq!(
        lowered.materialized().category(),
        FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(
        lowered.materialized().role(),
        FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
    assert_eq!(
        lowered.materialized().source(),
        FoundationalBoundaryMaterializationSource::NativeAuthority
    );
    assert_eq!(
        lowered.materialized().seam(),
        FoundationalBoundaryMaterializationSeam::BoundaryExchange
    );
    accepts_current_basis_proof(lowered.proofs());
}

#[test]
fn foundational_current_basis_bridge_and_readmission_preserve_exact_basis_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let basis = runtime
        .history()
        .resolve_merge_branch_basis(
            &BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("relational basis");
    let lowered = runtime
        .history()
        .lower_merge_branch_basis_to_foundational_current_basis(
            &basis,
            version("m7e.phase2.bridge"),
            materialized_profile(),
        )
        .expect("foundational current basis");
    let rebound_basis = lowered.strong_basis().clone();

    let bridged = lowered.bridge_trust_boundary();
    assert_exact_branch_basis(bridged.basis(), &basis);
    assert_eq!(
        bridged.materialized().source(),
        FoundationalBoundaryMaterializationSource::NativeAuthority
    );

    let readmitted = bridged.readmit_with_authority(
        rebound_basis,
        foundational_boundary_current_basis_readmission_authority(),
    );
    assert_exact_branch_basis(readmitted.basis(), &basis);
    assert_eq!(
        readmitted.strong_basis().payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    accepts_current_basis_proof(readmitted.proofs());
}

#[test]
fn foundational_current_basis_lowering_rejects_published_historical_basis_in_live_and_recovered_lanes(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            crate::facade::merge::MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_authority =
        published_merge_authority(execution, &runtime, merge.commit.commit.commit_id);

    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let recovered_authority =
        published_merge_authority(execution, &recovered, merge.commit.commit.commit_id);

    assert_historical_summary_basis_denies_as_current(&runtime, &live_authority);
    assert_historical_summary_basis_denies_as_current(&recovered, &recovered_authority);
}

#[test]
fn foundational_current_basis_lowering_rejects_stale_relational_basis_before_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let stale_basis = runtime
        .history()
        .resolve_merge_branch_basis(
            &BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("stale relational basis");

    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-drift",
        BranchId("feature".to_string()),
    );

    match runtime
        .history()
        .lower_merge_branch_basis_to_foundational_current_basis(
            &stale_basis,
            version("m7e.phase2.stale"),
            materialized_profile(),
        ) {
        Err(RelationalMergeBranchBasisFoundationalLoweringDenial::CurrentBasisDrift {
            retained_digest,
            current_digest,
        }) => {
            assert_eq!(retained_digest, stale_basis.basis_digest());
            assert_ne!(current_digest, retained_digest);
        }
        Ok(_) => panic!("stale basis must not lower as current basis"),
        Err(other) => panic!("unexpected lowering denial: {other:?}"),
    }
}

fn merge_ready_runtime(execution: &worth_execution::ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    runtime
}

fn materialized_profile() -> worth_foundational::MaterializedFoundationalProfileSet {
    let profile = FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::CertificationReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Durable,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile");
    let requested = request_foundational_profile_set(profile);
    let admitted = match worth_foundational::admit_requested_foundational_profile(
        requested,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        _ => panic!("expected admitted profile"),
    };

    match worth_foundational::materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        _ => panic!("expected materialized profile"),
    }
}

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid canonicalization version")
}

fn assert_historical_summary_basis_denies_as_current(
    runtime: &RelationalRuntime,
    authority: &PublishedMergeExecutionAuthority,
) {
    match runtime
        .history()
        .lower_merge_branch_basis_to_foundational_current_basis(
            &authority.execution_summary.branch_basis,
            version("m7e.phase2.recovered"),
            materialized_profile(),
        ) {
        Err(RelationalMergeBranchBasisFoundationalLoweringDenial::CurrentBasisDrift {
            retained_digest,
            current_digest,
        }) => {
            assert_eq!(
                retained_digest,
                authority.execution_summary.branch_basis.basis_digest()
            );
            assert_ne!(current_digest, retained_digest);
        }
        Ok(_) => panic!("historical published basis must not lower as current basis"),
        Err(other) => panic!("unexpected historical basis denial: {other:?}"),
    }
}

fn published_merge_authority(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: crate::history::data::CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}

fn assert_exact_branch_basis(
    actual: &RelationalMergeBranchBasis,
    expected: &RelationalMergeBranchBasis,
) {
    assert_eq!(actual, expected);
    assert_eq!(actual.basis_digest(), expected.basis_digest());
    assert_eq!(actual.source_branch(), expected.source_branch());
    assert_eq!(actual.target_branch(), expected.target_branch());
    assert_eq!(actual.source_head(), expected.source_head());
    assert_eq!(actual.target_head(), expected.target_head());
    assert_eq!(actual.merge_base().rule(), expected.merge_base().rule());
    assert_eq!(actual.merge_base().commit(), expected.merge_base().commit());
    assert_eq!(
        actual.merge_base().supporting_left_ancestors(),
        expected.merge_base().supporting_left_ancestors()
    );
    assert_eq!(
        actual.merge_base().supporting_right_ancestors(),
        expected.merge_base().supporting_right_ancestors()
    );
}

fn accepts_current_basis_proof(
    _: &Proof<FoundationalBoundaryCurrentBasisCertified, FoundationalBoundaryCurrentBasisAuthority>,
) {
}
