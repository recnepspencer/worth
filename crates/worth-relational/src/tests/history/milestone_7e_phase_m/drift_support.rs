use worth_relational_contracts::internal_port::merge::data::artifacts::correspondence_witness::RelationalMergeCorrespondenceWitnessOwnerPort as _;
use worth_relational_contracts::internal_port::merge::data::artifacts::proof_packet::RelationalMergeProofPacketOwnerPort as _;
use worth_relational_contracts::internal_port::merge::data::artifacts::schema_reconciliation_witness::RelationalSchemaReconciliationWitnessOwnerPort as _;
use worth_relational_contracts::internal_port::merge::data::artifacts::strategy_witness::RelationalMergeStrategyWitnessOwnerPort as _;
use std::sync::Arc;

use crate::facade::merge::{
    RelationalMergeCorrespondenceWitness, RelationalMergeProofPacket,
    RelationalMergeStrategyWitness, RelationalSchemaReconciliationWitness,
};
use crate::transactions::data::PublishedMergeExecutionAuthority;

pub(super) fn collaboration_family_drift(
    baseline: &PublishedMergeExecutionAuthority,
    candidate: &PublishedMergeExecutionAuthority,
) -> Vec<&'static str> {
    let mut drift = Vec::new();
    if baseline.execution_summary.request != candidate.execution_summary.request {
        drift.push("request");
    }
    if baseline.execution_summary.branch_basis != candidate.execution_summary.branch_basis {
        drift.push("branch_basis");
    }
    if baseline.execution_summary.proof_packet != candidate.execution_summary.proof_packet {
        drift.push("proof_packet");
    }
    if baseline.execution_summary.correspondence_witness
        != candidate.execution_summary.correspondence_witness
    {
        drift.push("correspondence_witness");
    }
    if baseline.execution_summary.schema_reconciliation_witness
        != candidate.execution_summary.schema_reconciliation_witness
    {
        drift.push("schema_reconciliation_witness");
    }
    if baseline.execution_summary.strategy_witness != candidate.execution_summary.strategy_witness {
        drift.push("strategy_witness");
    }
    drift
}

pub(super) fn branch_basis_drifted_authority(
    baseline: &PublishedMergeExecutionAuthority,
    branch_basis: crate::history::data::RelationalMergeBranchBasis,
) -> PublishedMergeExecutionAuthority {
    let mut drifted = baseline.clone();
    drifted.execution_summary.branch_basis = branch_basis;
    drifted
}

pub(super) fn proof_packet_drifted_authority(
    baseline: &PublishedMergeExecutionAuthority,
) -> PublishedMergeExecutionAuthority {
    let packet = baseline.execution_summary.proof_packet();
    let forged_packet = RelationalMergeProofPacket::retained_execution_admitted(
        packet.request().clone(),
        packet.branch_basis().clone(),
        Arc::from(packet.admitted_merge_surface().to_vec()),
        packet.correspondence_witness_digest().to_string(),
        packet.schema_reconciliation_witness_digest().to_string(),
        packet.strategy_witness_digest().to_string(),
        packet.foundational_request_lowering_digest().to_string(),
        "abababababababababababababababababababababababababababababababab".to_string(),
        packet.execution_digest().to_string(),
    );
    let mut drifted = baseline.clone();
    drifted.execution_summary.proof_packet = forged_packet;
    drifted
}

pub(super) fn correspondence_drifted_authority(
    baseline: &PublishedMergeExecutionAuthority,
) -> PublishedMergeExecutionAuthority {
    let summary = &baseline.execution_summary;
    let witness = RelationalMergeCorrespondenceWitness::retained(
        summary.request.request_digest().to_string(),
        summary.branch_basis.basis_digest(),
        Arc::from(Vec::<
            crate::merge::data::RelationalMergeCorrespondenceWitnessRow,
        >::new()),
    );
    let mut drifted = baseline.clone();
    drifted.execution_summary.correspondence_witness = witness;
    drifted
}

pub(super) fn schema_drifted_authority(
    baseline: &PublishedMergeExecutionAuthority,
) -> PublishedMergeExecutionAuthority {
    let summary = &baseline.execution_summary;
    let witness = RelationalSchemaReconciliationWitness::retained(
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef".to_string(),
        summary.branch_basis.basis_digest(),
        Arc::from(summary.schema_reconciliation_witness.rows().to_vec()),
    );
    let mut drifted = baseline.clone();
    drifted.execution_summary.schema_reconciliation_witness = witness;
    drifted
}

pub(super) fn strategy_drifted_authority(
    baseline: &PublishedMergeExecutionAuthority,
) -> PublishedMergeExecutionAuthority {
    let summary = &baseline.execution_summary;
    let witness = RelationalMergeStrategyWitness::retained(
        summary.request.request_digest().to_string(),
        summary.branch_basis.basis_digest(),
        summary
            .strategy_witness
            .execution_authority_contract()
            .clone(),
        Arc::from(Vec::<
            crate::merge::data::RelationalMergeAspectPolicyWitnessRow,
        >::new()),
        Arc::from(Vec::<
            crate::merge::data::RelationalMergeTopologyStrategyWitnessRow,
        >::new()),
        Arc::from(Vec::<
            crate::merge::data::RelationalMergeDeletionStrategyWitnessRow,
        >::new()),
    );
    let mut drifted = baseline.clone();
    drifted.execution_summary.strategy_witness = witness;
    drifted
}
