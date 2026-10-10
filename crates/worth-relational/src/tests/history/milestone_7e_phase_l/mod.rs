mod fixtures;

use crate::facade::inspection::{
    RelationalMergeSupportInspectionAbsenceKind,
    RelationalMergeSupportInspectionCompatibilityPosture, RelationalMergeSupportInspectionRow,
};
use crate::inspection::data::RelationalMergeSupportInspectionInput;
use crate::inspection::support_inspection_witness;
use crate::tests::support::checkpoint_and_recover_with;

use fixtures::{
    prepared_merge, published_merge_authority, runtime_with_schema_declared_entity_policy,
};

#[test]
fn merge_support_inspection_witness_parity_survives_live_replay_recovery_and_compatibility_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(true);
    let prepared = prepared_merge(execution, &runtime);
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_support = runtime
        .inspect_what_happened()
        .prepare_merge_support_inspection_witness(&outcome.execution_summary)
        .expect("live support");
    let published_authority = published_merge_authority(&runtime, outcome.commit.commit.commit_id);
    let published_support = runtime
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&published_authority)
        .expect("published support");
    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        runtime_with_schema_declared_entity_policy(true)
    });
    let recovered_authority =
        published_merge_authority(&recovered, outcome.commit.commit.commit_id);
    let recovered_support = recovered
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&recovered_authority)
        .expect("recovered support");

    assert_eq!(live_support, published_support);
    assert_eq!(published_support, recovered_support);
    assert_eq!(published_support.rows(), recovered_support.rows());
    assert_eq!(published_support.profile(), recovered_support.profile());
    assert_eq!(
        published_support.witness_digest(),
        recovered_support.witness_digest()
    );
    assert!(published_support.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Compatibility {
            posture: RelationalMergeSupportInspectionCompatibilityPosture::UnavailablePhaseDependency,
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitnessPhaseDependency
            ),
            ..
        }
    )));
    assert_ne!(live_support.witness_digest(), "");
}

#[test]
fn merge_support_inspection_witness_denies_inconsistent_retained_proof_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(false);
    let prepared = prepared_merge(execution, &runtime);
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let honest_summary = outcome.execution_summary.clone();
    let honest_packet = honest_summary.proof_packet.clone();
    let forged_packet =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            honest_packet.request().clone(),
            honest_packet.branch_basis().clone(),
            std::sync::Arc::from(honest_packet.admitted_merge_surface().to_vec()),
            honest_packet.correspondence_witness_digest().to_string(),
            honest_packet
                .schema_reconciliation_witness_digest()
                .to_string(),
            "f".repeat(64),
            honest_packet
                .foundational_request_lowering_digest()
                .to_string(),
            honest_packet.planning_digest().to_string(),
            honest_packet.execution_digest().to_string(),
        );
    let forged_summary = crate::transactions::data::MergeExecutionSummary {
        proof_packet: forged_packet,
        ..honest_summary
    };

    let denied = runtime
        .inspect_what_happened()
        .prepare_merge_support_inspection_witness(&forged_summary);
    assert!(matches!(
        denied,
        Err(crate::facade::inspection::RelationalMergeSupportInspectionDenial::InconsistentRetainedProofAuthority)
    ));
}

#[test]
fn merge_support_inspection_witness_denies_missing_required_proof_family() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy(false);
    let prepared = prepared_merge(execution, &runtime);
    let correspondence = runtime
        .merge()
        .retain_merge_correspondence_witness_from_prepared_execution(&prepared);
    let input = RelationalMergeSupportInspectionInput {
        request: prepared.request().clone(),
        branch_basis: prepared.execution_ready_plan().basis.clone(),
        proof_packet: None,
        correspondence_witness: Some(correspondence.clone()),
        schema_reconciliation_witness: Some(
            prepared.artifact().schema_reconciliation_witness.clone(),
        ),
        strategy_witness: Some(prepared.artifact().strategy_witness.clone()),
    };
    let missing_packet = support_inspection_witness(input.clone()).expect("missing packet support");
    let missing_correspondence =
        support_inspection_witness(RelationalMergeSupportInspectionInput {
            correspondence_witness: None,
            ..input.clone()
        })
        .expect("missing correspondence support");
    let missing_schema = support_inspection_witness(RelationalMergeSupportInspectionInput {
        schema_reconciliation_witness: None,
        ..input.clone()
    })
    .expect("missing schema support");
    let missing_strategy = support_inspection_witness(RelationalMergeSupportInspectionInput {
        strategy_witness: None,
        ..input
    })
    .expect("missing strategy support");

    assert!(missing_packet.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::RequestAdmission {
            absence: Some(RelationalMergeSupportInspectionAbsenceKind::MissingProofPacket),
            ..
        }
    )));
    assert!(missing_correspondence.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Correspondence {
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCorrespondenceWitness
            ),
            ..
        }
    )));
    assert!(missing_schema.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Schema {
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingSchemaReconciliationWitness
            ),
            ..
        }
    )));
    assert!(missing_strategy.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Strategy {
            absence: Some(RelationalMergeSupportInspectionAbsenceKind::MissingStrategyWitness),
            ..
        }
    )));
}
