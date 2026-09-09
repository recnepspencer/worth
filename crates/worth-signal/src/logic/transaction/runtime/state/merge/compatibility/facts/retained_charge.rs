use super::SignalMergeCompatibilityFactInventory;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for SignalMergeCompatibilityFactInventory {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            branch_id: _,
            snapshot_id: _,
            branch_basis_digest,
            branch_head_posture,
            branch_restore_posture,
            scope_family: _,
            declaration_digest,
            admitted_scope_digest,
            skipped_scope_digest,
            no_op_scope_digest,
            breadth_summary,
            strategy_witness_digest,
            merge_strategy_digest,
            invalidation_strategy_digest,
            delivery_strategy_digest,
        } = self;
        branch_basis_digest
            .retained_heap_charge(work)?
            .checked_add(branch_head_posture.retained_heap_charge(work)?)?
            .checked_add(branch_restore_posture.retained_heap_charge(work)?)?
            .checked_add(declaration_digest.retained_heap_charge(work)?)?
            .checked_add(admitted_scope_digest.retained_heap_charge(work)?)?
            .checked_add(skipped_scope_digest.retained_heap_charge(work)?)?
            .checked_add(no_op_scope_digest.retained_heap_charge(work)?)?
            .checked_add(breadth_summary.retained_heap_charge(work)?)?
            .checked_add(strategy_witness_digest.retained_heap_charge(work)?)?
            .checked_add(merge_strategy_digest.retained_heap_charge(work)?)?
            .checked_add(invalidation_strategy_digest.retained_heap_charge(work)?)?
            .checked_add(delivery_strategy_digest.retained_heap_charge(work)?)
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
    #[test]
    fn replay_owned_digests_each_contribute_their_capacity() {
        let event = emitted_merge_replay_event();
        let mut value = event
            .detail
            .as_ref()
            .unwrap()
            .as_compatibility_witness()
            .unwrap()
            .fact_inventory()
            .clone();
        let original = value.clone();
        let before = value.retained_heap_charge(&mut Work::new(10000)).unwrap();
        let mut delta = 0;
        for field in [
            &mut value.branch_basis_digest,
            &mut value.declaration_digest,
            &mut value.admitted_scope_digest,
            &mut value.strategy_witness_digest,
            &mut value.merge_strategy_digest,
            &mut value.invalidation_strategy_digest,
            &mut value.delivery_strategy_digest,
        ] {
            delta += grow_string(field);
        }
        for field in [
            &mut value.skipped_scope_digest,
            &mut value.no_op_scope_digest,
        ]
        .into_iter()
        .flatten()
        {
            delta += grow_string(field);
        }
        assert_eq!(
            value
                .retained_heap_charge(&mut Work::new(10000))
                .unwrap()
                .bytes()
                - before.bytes(),
            delta
        );
        assert_eq!(value, original);
    }
}
