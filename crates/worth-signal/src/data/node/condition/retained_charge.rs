use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{EvaluationCondition, NodeEvaluationConfig};

impl RetainedStorageMeasurement for EvaluationCondition {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Custom(key) => key.retained_heap_charge(work),
            Self::Temporal(condition) => condition.retained_heap_charge(work),
            Self::Always
            | Self::AspectFilter(_)
            | Self::DeltaThreshold(_)
            | Self::OnDemand
            | Self::Installed(_) => Ok(Charge::ZERO),
        }
    }
}

impl RetainedStorageMeasurement for NodeEvaluationConfig {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            schema_binding,
            merge_strategy_name,
            conflict_policy_name,
            identity_matcher_name,
            source_only_policy_name,
            deletion_policy_name,
            conflict_isolation_policy_name,
            aspect_merge_policy_bindings,
            contract,
            condition,
            comparator,
            output_equivalence,
            partitioned_output: _,
        } = self;
        Ok(Charge::ZERO
            .checked_add(schema_binding.retained_heap_charge(work)?)?
            .checked_add(merge_strategy_name.retained_heap_charge(work)?)?
            .checked_add(conflict_policy_name.retained_heap_charge(work)?)?
            .checked_add(identity_matcher_name.retained_heap_charge(work)?)?
            .checked_add(source_only_policy_name.retained_heap_charge(work)?)?
            .checked_add(deletion_policy_name.retained_heap_charge(work)?)?
            .checked_add(conflict_isolation_policy_name.retained_heap_charge(work)?)?
            .checked_add(aspect_merge_policy_bindings.retained_heap_charge(work)?)?
            .checked_add(contract.retained_heap_charge(work)?)?
            .checked_add(condition.retained_heap_charge(work)?)?
            .checked_add(comparator.retained_heap_charge(work)?)?
            .checked_add(output_equivalence.retained_heap_charge(work)?)?)
    }
}
