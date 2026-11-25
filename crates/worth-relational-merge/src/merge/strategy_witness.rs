use crate::contracts::merge::data::artifacts::strategy_witness::RelationalMergeStrategyWitnessOwnerPort as _;
use crate::contracts::merge::data::artifacts::strategy_witness_policy_rows::RelationalMergeAspectPolicyWitnessRowOwnerPort as _;
use crate::contracts::merge::data::artifacts::strategy_witness_posture_rows::{
    RelationalMergeDeletionStrategyWitnessRowOwnerPort as _,
    RelationalMergeTopologyStrategyWitnessRowOwnerPort as _,
};
use std::sync::Arc;

use crate::merge::data::{
    LoweredMergePlan, MergeExecutionAuthorityContract, RelationalMergeAspectPolicyWitnessRow,
    RelationalMergeDeletionStrategyWitnessRow, RelationalMergeStrategyWitness,
    RelationalMergeTopologyStrategyWitnessRow,
};

pub(crate) fn retained_merge_strategy_witness(
    plan: &LoweredMergePlan,
    execution_authority_contract: &MergeExecutionAuthorityContract,
) -> RelationalMergeStrategyWitness {
    let aspect_policy_rows = plan
        .policy_records
        .iter()
        .map(|record| {
            RelationalMergeAspectPolicyWitnessRow::retained(
                record.record.clone(),
                record.target_record.clone(),
                record.classification,
                record.aspect_resolutions.clone(),
                record.applied_policies.clone(),
                record.proof_boundary,
            )
        })
        .collect::<Vec<_>>();
    let topology_rows = plan
        .lowered_records
        .iter()
        .filter_map(|record| match record.resolution_class {
            crate::merge::data::MergeResolutionClass::Topology(topology_class) => {
                Some(RelationalMergeTopologyStrategyWitnessRow::retained(
                    record.record.clone(),
                    record.target_record.clone(),
                    topology_class,
                    record.readiness,
                    record.blocked_reason,
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let deletion_rows = plan
        .lowered_records
        .iter()
        .filter_map(|record| match record.resolution_class {
            crate::merge::data::MergeResolutionClass::Deletion(deletion_class) => {
                Some(RelationalMergeDeletionStrategyWitnessRow::retained(
                    record.record.clone(),
                    record.target_record.clone(),
                    deletion_class,
                    record.readiness,
                    record.blocked_reason,
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    RelationalMergeStrategyWitness::retained(
        plan.request.request_digest().to_string(),
        plan.basis.basis_digest(),
        execution_authority_contract.clone(),
        Arc::from(aspect_policy_rows),
        Arc::from(topology_rows),
        Arc::from(deletion_rows),
    )
}
