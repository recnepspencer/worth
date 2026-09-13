use super::{
    SignalAspectPolicyInventoryEntry, SignalDeliveryStrategyIdentity,
    SignalInvalidationStrategyIdentity, SignalMergeStrategyIdentity,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalAspectPolicyInventoryEntry {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            policy_name,
            policy_digest,
            policy_basis: _,
        } = self;
        policy_name
            .retained_heap_charge(work)?
            .checked_add(policy_digest.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for SignalMergeStrategyIdentity {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            merge_strategy: _,
            selected_strategy_name,
            selected_strategy_digest,
            selected_strategy_basis: _,
            merge_base_name,
            merge_base_digest,
            merge_base_basis: _,
            lowered_strategy_bundle_digest,
        } = self;
        selected_strategy_name
            .retained_heap_charge(work)?
            .checked_add(selected_strategy_digest.retained_heap_charge(work)?)?
            .checked_add(merge_base_name.retained_heap_charge(work)?)?
            .checked_add(merge_base_digest.retained_heap_charge(work)?)?
            .checked_add(lowered_strategy_bundle_digest.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for SignalInvalidationStrategyIdentity {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            boundary_witness_kind: _,
            conflict_isolation_name,
            conflict_isolation_digest,
            conflict_isolation_basis: _,
            identity_matcher_name,
            identity_matcher_digest,
            identity_matcher_basis: _,
        } = self;
        conflict_isolation_name
            .retained_heap_charge(work)?
            .checked_add(conflict_isolation_digest.retained_heap_charge(work)?)?
            .checked_add(identity_matcher_name.retained_heap_charge(work)?)?
            .checked_add(identity_matcher_digest.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for SignalDeliveryStrategyIdentity {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            conflict_policy_name,
            conflict_policy_digest,
            conflict_policy_basis: _,
            source_only_policy_name,
            source_only_policy_digest,
            source_only_policy_basis: _,
            deletion_policy_name,
            deletion_policy_digest,
            deletion_policy_basis: _,
            aspect_policy_inventory,
            runtime_artifact_carry_policies,
            retained_artifact_carry_policies,
            causality_carry_policies,
        } = self;
        conflict_policy_name
            .retained_heap_charge(work)?
            .checked_add(conflict_policy_digest.retained_heap_charge(work)?)?
            .checked_add(source_only_policy_name.retained_heap_charge(work)?)?
            .checked_add(source_only_policy_digest.retained_heap_charge(work)?)?
            .checked_add(deletion_policy_name.retained_heap_charge(work)?)?
            .checked_add(deletion_policy_digest.retained_heap_charge(work)?)?
            .checked_add(aspect_policy_inventory.retained_heap_charge(work)?)?
            .checked_add(runtime_artifact_carry_policies.retained_heap_charge(work)?)?
            .checked_add(retained_artifact_carry_policies.retained_heap_charge(work)?)?
            .checked_add(causality_carry_policies.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::emitted_merge_replay_event;
    fn grow_string(value: &mut String) -> u64 {
        let before = value.capacity();
        value.reserve_exact(1024);
        (value.capacity() - before) as u64
    }
    fn grow_vec<T>(values: &mut Vec<T>) -> u64 {
        let before = values.capacity();
        values.reserve_exact(64);
        ((values.capacity() - before) * std::mem::size_of::<T>()) as u64
    }
    #[test]
    fn replay_strategy_identities_charge_digests_and_nested_inventories() {
        let event = emitted_merge_replay_event();
        let witness = event
            .detail
            .as_ref()
            .unwrap()
            .as_strategy_witness()
            .unwrap();
        let mut merge = witness.merge_strategy().clone();
        let before = merge.retained_heap_charge(&mut Work::new(10000)).unwrap();
        let delta = grow_string(&mut merge.selected_strategy_digest)
            + grow_string(&mut merge.merge_base_digest)
            + grow_string(&mut merge.lowered_strategy_bundle_digest);
        assert_eq!(
            merge
                .retained_heap_charge(&mut Work::new(10000))
                .unwrap()
                .bytes()
                - before.bytes(),
            delta
        );
        assert_eq!(&merge, witness.merge_strategy());
        let mut invalidation = witness.invalidation_strategy().clone();
        let before = invalidation
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap();
        let delta = grow_string(&mut invalidation.conflict_isolation_digest)
            + grow_string(&mut invalidation.identity_matcher_digest);
        assert_eq!(
            invalidation
                .retained_heap_charge(&mut Work::new(10000))
                .unwrap()
                .bytes()
                - before.bytes(),
            delta
        );
        assert_eq!(&invalidation, witness.invalidation_strategy());
        let mut delivery = witness.delivery_strategy().clone();
        let before = delivery
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap();
        let mut delta = grow_string(&mut delivery.conflict_policy_digest)
            + grow_string(&mut delivery.source_only_policy_digest)
            + grow_string(&mut delivery.deletion_policy_digest);
        delta += grow_vec(&mut delivery.aspect_policy_inventory);
        delta += grow_vec(&mut delivery.runtime_artifact_carry_policies);
        delta += grow_vec(&mut delivery.retained_artifact_carry_policies);
        delta += grow_vec(&mut delivery.causality_carry_policies);
        for entry in &mut delivery.aspect_policy_inventory {
            delta += grow_string(&mut entry.policy_digest);
        }
        assert_eq!(
            delivery
                .retained_heap_charge(&mut Work::new(10000))
                .unwrap()
                .bytes()
                - before.bytes(),
            delta
        );
        assert_eq!(&delivery, witness.delivery_strategy());
    }
    #[test]
    fn policy_inventory_entry_charges_name_and_digest_spare_capacity() {
        // Representation fixture, never installed policy authority.
        let mut name = String::with_capacity(512);
        name.push_str("policy");
        let mut digest = String::with_capacity(1024);
        digest.push_str("digest");
        let expected = (name.capacity() + digest.capacity()) as u64;
        let entry = SignalAspectPolicyInventoryEntry::new(
            super::super::AspectMergePolicyName::new(name),
            digest,
            super::super::AspectMergePolicySelectionBasis::BuiltInDefault,
        );
        assert_eq!(
            entry
                .retained_heap_charge(&mut Work::new(100))
                .unwrap()
                .bytes(),
            expected
        );
    }
}
