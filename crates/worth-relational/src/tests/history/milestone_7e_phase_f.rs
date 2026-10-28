use std::collections::BTreeMap;
use std::sync::Arc;

use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeExecutionOutcome, MergeExecutionRequest, MergeIntent,
    RelationalMergeProofPacketCanonicalBasis,
};
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{
    checkpoint_and_recover_with, create_branch_from_main, create_entity,
    create_entity_outcome_on_branch, durable_runtime_with_test_schema,
};
use worth_foundational::{CanonicalBasisLocus, CanonicalBasisValue, InternedString};

#[test]
fn merge_proof_packet_canonical_basis_parity_survives_publication_and_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let outcome = execute_merge(execution, &runtime);
    let live_packet = outcome.execution_summary.proof_packet().clone();
    let live_basis = lower_packet(&runtime, &live_packet);
    let published_packet =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id)
            .execution_summary
            .proof_packet
            .clone();
    let published_basis = lower_packet(&runtime, &published_packet);
    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let recovered_packet =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id)
            .execution_summary
            .proof_packet
            .clone();
    let recovered_basis = lower_packet(&recovered, &recovered_packet);

    assert_eq!(live_basis, published_basis);
    assert_eq!(live_basis, recovered_basis);
    assert_eq!(entry_map(&live_basis), entry_map(&published_basis));
    assert_eq!(entry_map(&live_basis), entry_map(&recovered_basis));
}

#[test]
fn merge_proof_packet_canonical_basis_localizes_single_field_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let packet = runtime
        .merge()
        .retain_merge_proof_packet_from_prepared_execution(&prepared);
    let baseline = entry_map(&lower_packet(&runtime, &packet));

    let drifted_request =
        crate::facade::merge::NormalizedRelationalMergeRequest::admit_full_branch(
            BranchId("main".to_string()),
            BranchId("alt".to_string()),
            MergeIntent::ReconcileIntoTarget,
            crate::facade::merge::RelationalMergeCorrespondencePosture::Advisory,
            crate::facade::merge::RelationalMergeSchemaReconciliationPosture::Participate,
            crate::facade::merge::RelationalMergeTopologyIntent::PreserveTopologySemantics,
        )
        .expect("admitted alternate normalized request");
    let drifted_request_lowering = runtime
        .merge()
        .lower_merge_request_to_foundational(drifted_request.clone());
    let worth_foundational::FoundationalMergeAdmissionOutcome::Success(drifted_request_lowering) =
        drifted_request_lowering
    else {
        panic!("alternate normalized request must lower to foundational merge vocabulary");
    };
    let request_drifted_packet =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            drifted_request,
            packet.branch_basis().clone(),
            Arc::from(packet.admitted_merge_surface().to_vec()),
            packet.correspondence_witness_digest().to_string(),
            packet.schema_reconciliation_witness_digest().to_string(),
            packet.strategy_witness_digest().to_string(),
            drifted_request_lowering.lowering_digest().to_string(),
            packet.planning_digest().to_string(),
            packet.execution_digest().to_string(),
        );
    let request_drift = differing_loci(
        &baseline,
        &entry_map(&lower_packet(&runtime, &request_drifted_packet)),
    );
    assert_eq!(
        request_drift,
        vec![
            "merge.packet.digest",
            "merge.request.digest",
            "merge.request_lowering.digest",
        ]
    );

    let branch_basis_drifted = runtime
        .history()
        .resolve_merge_branch_basis(&BranchId("alt".to_string()), &BranchId("main".to_string()))
        .expect("alternate branch basis");
    let branch_basis_drifted_packet =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            packet.request().clone(),
            branch_basis_drifted,
            Arc::from(packet.admitted_merge_surface().to_vec()),
            packet.correspondence_witness_digest().to_string(),
            packet.schema_reconciliation_witness_digest().to_string(),
            packet.strategy_witness_digest().to_string(),
            packet.foundational_request_lowering_digest().to_string(),
            packet.planning_digest().to_string(),
            packet.execution_digest().to_string(),
        );
    let branch_basis_drift = differing_loci(
        &baseline,
        &entry_map(&lower_packet(&runtime, &branch_basis_drifted_packet)),
    );
    assert_eq!(
        branch_basis_drift,
        vec!["merge.branch_basis.digest", "merge.packet.digest"]
    );

    let planning_drifted_packet =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            packet.request().clone(),
            packet.branch_basis().clone(),
            Arc::from(packet.admitted_merge_surface().to_vec()),
            packet.correspondence_witness_digest().to_string(),
            packet.schema_reconciliation_witness_digest().to_string(),
            packet.strategy_witness_digest().to_string(),
            packet.foundational_request_lowering_digest().to_string(),
            "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd".to_string(),
            packet.execution_digest().to_string(),
        );
    let planning_drift = differing_loci(
        &baseline,
        &entry_map(&lower_packet(&runtime, &planning_drifted_packet)),
    );
    assert_eq!(
        planning_drift,
        vec!["merge.packet.digest", "merge.planning.digest"]
    );

    let drifted_execution =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            packet.request().clone(),
            packet.branch_basis().clone(),
            Arc::from(packet.admitted_merge_surface().to_vec()),
            packet.correspondence_witness_digest().to_string(),
            packet.schema_reconciliation_witness_digest().to_string(),
            packet.strategy_witness_digest().to_string(),
            packet.foundational_request_lowering_digest().to_string(),
            packet.planning_digest().to_string(),
            "abababababababababababababababababababababababababababababababab".to_string(),
        );
    let execution_drift = differing_loci(
        &baseline,
        &entry_map(&lower_packet(&runtime, &drifted_execution)),
    );
    assert_eq!(
        execution_drift,
        vec!["merge.execution.digest", "merge.packet.digest"]
    );

    let mut drifted_rows = packet.admitted_merge_surface().to_vec();
    drifted_rows[0] = crate::facade::merge::RelationalMergeAdmittedSurfaceRow::new(
        crate::transactions::data::RecordRef::Entity(crate::identity::data::EntityId::new(
            crate::identity::data::PartitionId::main(),
            777,
            1,
        )),
        None,
    );
    let drifted_surface =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            packet.request().clone(),
            packet.branch_basis().clone(),
            Arc::from(drifted_rows),
            packet.correspondence_witness_digest().to_string(),
            packet.schema_reconciliation_witness_digest().to_string(),
            packet.strategy_witness_digest().to_string(),
            packet.foundational_request_lowering_digest().to_string(),
            packet.planning_digest().to_string(),
            packet.execution_digest().to_string(),
        );
    let surface_drift = differing_loci(
        &baseline,
        &entry_map(&lower_packet(&runtime, &drifted_surface)),
    );
    assert_eq!(
        surface_drift,
        vec![
            "merge.admitted_surface.digest",
            "merge.admitted_surface.row.0.digest",
            "merge.packet.digest",
        ]
    );
}

fn merge_ready_runtime(execution: &worth_execution::ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_branch_from_main(&runtime, "alt");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    create_entity_outcome_on_branch(execution, &runtime, "alt-only", BranchId("alt".to_string()));
    runtime
}

fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}

fn execute_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergeExecutionOutcome {
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge")
}

fn published_merge_authority(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> crate::transactions::data::PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}

fn lower_packet(
    runtime: &RelationalRuntime,
    packet: &crate::facade::merge::RelationalMergeProofPacket,
) -> RelationalMergeProofPacketCanonicalBasis {
    let outcome = runtime
        .merge()
        .lower_merge_proof_packet_to_foundational_canonical_basis(packet);
    let worth_proof::TransitionOutcome::Success(basis) = outcome else {
        panic!("phase 6 canonical lowering must succeed for retained merge proof packet");
    };
    basis
}

fn entry_map(basis: &RelationalMergeProofPacketCanonicalBasis) -> BTreeMap<String, String> {
    basis
        .entries()
        .iter()
        .map(|entry| {
            (
                entry_locus(entry.locus()),
                canonical_value_text(entry.value()),
            )
        })
        .collect()
}

fn entry_locus(locus: &CanonicalBasisLocus) -> String {
    match locus {
        CanonicalBasisLocus::Named(InternedString::Raw(value)) => value.clone(),
        CanonicalBasisLocus::Named(InternedString::Symbol(symbol)) => {
            format!("symbol:{}", symbol.0)
        }
        other => format!("{other:?}"),
    }
}

fn differing_loci(
    baseline: &BTreeMap<String, String>,
    candidate: &BTreeMap<String, String>,
) -> Vec<String> {
    baseline
        .iter()
        .filter_map(|(locus, baseline_value)| {
            let candidate_value = candidate.get(locus)?;
            (candidate_value != baseline_value).then_some(locus.clone())
        })
        .collect()
}

fn canonical_value_text(value: &CanonicalBasisValue) -> String {
    match value {
        CanonicalBasisValue::ExactText(InternedString::Raw(text)) => format!("text:{text}"),
        CanonicalBasisValue::ExactText(InternedString::Symbol(symbol)) => {
            format!("text:symbol:{}", symbol.0)
        }
        CanonicalBasisValue::BytesDigest(digest) => {
            let hex = digest
                .bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            format!("digest:{hex}")
        }
        CanonicalBasisValue::UnsignedInteger { value, .. } => format!("u128:{value}"),
        other => format!("unexpected:{other:?}"),
    }
}
