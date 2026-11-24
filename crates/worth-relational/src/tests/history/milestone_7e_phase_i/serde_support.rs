use worth_relational_contracts::internal_port::merge::data::artifacts::strategy_witness_policy_rows::RelationalMergeAspectPolicyWitnessRowOwnerPort as _;
use worth_relational_contracts::internal_port::merge::data::artifacts::strategy_witness_posture_rows::RelationalMergeDeletionStrategyWitnessRowOwnerPort as _;
use worth_relational_contracts::internal_port::merge::data::artifacts::strategy_witness_posture_rows::RelationalMergeTopologyStrategyWitnessRowOwnerPort as _;
use crate::facade::merge::{
    RelationalMergeAspectPolicyWitnessRow, RelationalMergeDeletionStrategyWitnessRow,
    RelationalMergeStrategyWitness, RelationalMergeTopologyStrategyWitnessRow,
};
use sha2::{Digest, Sha256};

#[derive(Default)]
pub(super) struct StrategyWitnessPayloadMutator<'a> {
    pub witness_digest: Option<&'a str>,
    pub execution_authority_contract: Option<crate::facade::merge::MergeExecutionAuthorityContract>,
    pub topology_readiness: Option<crate::facade::merge::MergeExecutionReadiness>,
    pub topology_class: Option<crate::facade::merge::TopologyExecutionClass>,
    pub deletion_class: Option<crate::facade::merge::DeletionExecutionClass>,
    pub policy_kind: Option<crate::facade::merge::AspectMergePolicyKind>,
}

#[derive(serde::Serialize)]
pub(super) struct StrategyWitnessPayload<'a> {
    request_digest: &'a str,
    branch_basis_digest: &'a str,
    execution_authority_contract: crate::facade::merge::MergeExecutionAuthorityContract,
    aspect_policy_rows: Vec<RelationalMergeAspectPolicyWitnessRow>,
    topology_rows: Vec<RelationalMergeTopologyStrategyWitnessRow>,
    deletion_rows: Vec<RelationalMergeDeletionStrategyWitnessRow>,
    witness_digest: String,
}

pub(super) fn strategy_witness_payload<'a>(
    witness: &'a RelationalMergeStrategyWitness,
    mutator: StrategyWitnessPayloadMutator<'a>,
) -> StrategyWitnessPayload<'a> {
    let execution_authority_contract = mutator
        .execution_authority_contract
        .unwrap_or_else(|| witness.execution_authority_contract().clone());
    let mut aspect_policy_rows = witness.aspect_policy_rows().to_vec();
    if let Some(policy_kind) = mutator.policy_kind {
        let row = &aspect_policy_rows[0];
        aspect_policy_rows[0] = RelationalMergeAspectPolicyWitnessRow::retained(
            row.record().clone(),
            row.target_record().cloned(),
            row.classification(),
            std::sync::Arc::from([crate::facade::merge::AspectPolicyResolutionRecord {
                aspect_key: row.aspect_resolutions()[0].aspect_key.clone(),
                comparison: row.aspect_resolutions()[0].comparison,
                applied_policy: Some(policy_kind.clone()),
                decision_boundary: row.aspect_resolutions()[0].decision_boundary,
                resolved_value_strategy: row.aspect_resolutions()[0]
                    .resolved_value_strategy
                    .clone(),
            }]),
            std::sync::Arc::from([crate::facade::merge::ResolvedAspectMergePolicy {
                aspect_key: row.applied_policies()[0].aspect_key.clone(),
                policy: policy_kind,
            }]),
            row.proof_boundary(),
        );
    }
    let mut topology_rows = witness.topology_rows().to_vec();
    if let Some(topology_class) = mutator.topology_class {
        let row = &topology_rows[0];
        topology_rows[0] = RelationalMergeTopologyStrategyWitnessRow::retained(
            row.record().clone(),
            row.target_record().cloned(),
            topology_class,
            mutator.topology_readiness.unwrap_or(row.readiness()),
            row.blocked_reason(),
        );
    }
    let mut deletion_rows = witness.deletion_rows().to_vec();
    if let Some(deletion_class) = mutator.deletion_class {
        let row = &deletion_rows[0];
        deletion_rows[0] = RelationalMergeDeletionStrategyWitnessRow::retained(
            row.record().clone(),
            row.target_record().cloned(),
            deletion_class,
            row.readiness(),
            row.blocked_reason(),
        );
    }
    let witness_digest = mutator
        .witness_digest
        .map(str::to_string)
        .unwrap_or_else(|| {
            strategy_witness_digest(
                witness.request_digest(),
                witness.branch_basis_digest(),
                &execution_authority_contract,
                &aspect_policy_rows,
                &topology_rows,
                &deletion_rows,
            )
        });
    StrategyWitnessPayload {
        request_digest: witness.request_digest(),
        branch_basis_digest: witness.branch_basis_digest(),
        execution_authority_contract,
        aspect_policy_rows,
        topology_rows,
        deletion_rows,
        witness_digest,
    }
}

fn strategy_witness_digest(
    request_digest: &str,
    branch_basis_digest: &str,
    execution_authority_contract: &crate::facade::merge::MergeExecutionAuthorityContract,
    aspect_policy_rows: &[RelationalMergeAspectPolicyWitnessRow],
    topology_rows: &[RelationalMergeTopologyStrategyWitnessRow],
    deletion_rows: &[RelationalMergeDeletionStrategyWitnessRow],
) -> String {
    let digest = Sha256::digest(
        rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.v1",
            request_digest,
            branch_basis_digest,
            execution_authority_contract,
            aspect_policy_rows,
            topology_rows,
            deletion_rows,
        ))
        .expect("strategy witness payload must encode"),
    );
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
