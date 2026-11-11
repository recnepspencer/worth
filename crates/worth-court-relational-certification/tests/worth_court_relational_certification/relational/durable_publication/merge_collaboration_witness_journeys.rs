use super::merge_proof_journey::published_merge_authority;
use super::merge_witness_authoring::*;
use super::merge_witness_schema::policy_builder;
use worth_relational::facade::merge::*;
use worth_relational::facade::{
    inspection::*,
    runtime::RelationalRuntime,
    transactions::{MergeExecutionOutcome, PublishedMergeExecutionAuthority},
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn merge_collaboration_truth_replay_equivalence_survives_live_publication_and_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let builder = policy_builder(AspectMergePolicyKind::PreferRicher, true);
    let runtime = builder.clone().build_durable(medium.attachment()).unwrap();
    let prepared = prepared_collaboration(execution, &runtime, true);
    let outcome = runtime.execute_prepared_merge(execution, prepared).unwrap();
    let live = snapshot_from_outcome(&runtime, &outcome);
    let published = snapshot_from_authority(
        &runtime,
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id),
    );
    super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
    let (_, recovered) = super::branch_history_recovery::recover(
        execution,
        &medium,
        builder.build_durable(medium.attachment()).unwrap(),
    );
    let recovered_snapshot = snapshot_from_authority(
        &recovered,
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id),
    );

    assert_eq!(
        live.authority.execution_summary,
        published.authority.execution_summary
    );
    assert_eq!(
        live.authority.execution_summary,
        recovered_snapshot.authority.execution_summary
    );
    assert_eq!(
        live.authority.execution_summary.request,
        published.authority.execution_summary.request
    );
    assert_eq!(
        live.authority.execution_summary.branch_basis,
        published.authority.execution_summary.branch_basis
    );
    assert_eq!(
        live.authority.execution_summary.correspondence_witness,
        published.authority.execution_summary.correspondence_witness
    );
    assert_eq!(
        live.authority
            .execution_summary
            .schema_reconciliation_witness,
        published
            .authority
            .execution_summary
            .schema_reconciliation_witness
    );
    assert_eq!(
        live.authority.execution_summary.strategy_witness,
        published.authority.execution_summary.strategy_witness
    );
    assert_eq!(live.canonical_basis, published.canonical_basis);
    assert_eq!(
        published.canonical_basis,
        recovered_snapshot.canonical_basis
    );
    assert_eq!(live.support, published.support);
    assert_eq!(published.support, recovered_snapshot.support);
    assert!(published.support.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Compatibility {
            posture: RelationalMergeSupportInspectionCompatibilityPosture::UnavailablePhaseDependency,
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitnessPhaseDependency
            ),
            ..
        }
    )));
}
#[test]
fn merge_collaboration_support_surface_reflects_correspondence_posture_over_real_history() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let builder = policy_builder(AspectMergePolicyKind::PreferRicher, true);
    let runtime = builder.clone().build_durable(medium.attachment()).unwrap();
    let prepared = prepared_collaboration(execution, &runtime, true);
    let outcome = runtime.execute_prepared_merge(execution, prepared).unwrap();
    let snapshot = snapshot_from_authority(
        &runtime,
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id),
    );
    let witness = &snapshot.authority.execution_summary.correspondence_witness;
    let row = snapshot
        .support
        .rows()
        .iter()
        .find_map(|row| match row {
            RelationalMergeSupportInspectionRow::Correspondence {
                witness_digest,
                admitted_count,
                denied_count,
                unavailable_count,
                sample_posture,
                ..
            } => Some((
                witness_digest.as_deref(),
                *admitted_count,
                *denied_count,
                *unavailable_count,
                *sample_posture,
            )),
            _ => None,
        })
        .expect("correspondence support row");

    let denied_count = witness
        .rows()
        .iter()
        .filter(|row| {
            !matches!(
                row.posture(),
                worth_relational::facade::merge::RelationalMergeCorrespondenceWitnessPosture::Admitted
                    | worth_relational::facade::merge::RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            )
        })
        .count();
    let unavailable_count = witness
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.posture(),
                worth_relational::facade::merge::RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            )
        })
        .count();

    assert_eq!(row.0, Some(witness.witness_digest()));
    assert_eq!(row.1, witness.admitted_rows().count());
    assert_eq!(row.2, denied_count);
    assert_eq!(row.3, unavailable_count);
    assert_eq!(row.4, witness.rows().first().map(|entry| entry.posture()));
}
struct CollaborationTruthSnapshot {
    authority: PublishedMergeExecutionAuthority,
    support: RelationalMergeSupportInspectionWitness,
    canonical_basis: RelationalMergeProofPacketCanonicalBasis,
}
fn snapshot_from_outcome(
    runtime: &RelationalRuntime,
    outcome: &MergeExecutionOutcome,
) -> CollaborationTruthSnapshot {
    snapshot_from_authority(
        runtime,
        PublishedMergeExecutionAuthority {
            execution_summary: outcome.execution_summary.clone(),
            structural_summary: outcome.structural_summary.clone(),
        },
    )
}
fn snapshot_from_authority(
    runtime: &RelationalRuntime,
    authority: PublishedMergeExecutionAuthority,
) -> CollaborationTruthSnapshot {
    let support = runtime
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&authority)
        .expect("support inspection witness");
    let canonical_basis = lower_packet(runtime, authority.execution_summary.proof_packet());
    CollaborationTruthSnapshot {
        authority,
        support,
        canonical_basis,
    }
}
fn lower_packet(
    runtime: &RelationalRuntime,
    packet: &worth_relational::facade::merge::RelationalMergeProofPacket,
) -> RelationalMergeProofPacketCanonicalBasis {
    let outcome = runtime
        .merge()
        .lower_merge_proof_packet_to_foundational_canonical_basis(packet);
    let worth_proof::TransitionOutcome::Success(basis) = outcome else {
        panic!("phase 13 canonical lowering must succeed for retained merge proof packet");
    };
    basis
}
