use super::ScopedMergeProofPacket;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ScopedMergeProofPacket {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            scope_family: _,
            declaration_digest,
            admitted_scope_digest,
            skipped_scope_digest,
            no_op_scope_digest,
            breadth_summary,
            requested_nodes,
            requested_aspects,
            admitted_nodes,
            admitted_aspects,
            skipped_nodes,
            skipped_aspects,
            no_op_nodes,
            no_op_aspects,
            support_closure_nodes,
        } = self;
        declaration_digest
            .retained_heap_charge(work)?
            .checked_add(admitted_scope_digest.retained_heap_charge(work)?)?
            .checked_add(skipped_scope_digest.retained_heap_charge(work)?)?
            .checked_add(no_op_scope_digest.retained_heap_charge(work)?)?
            .checked_add(breadth_summary.retained_heap_charge(work)?)?
            .checked_add(requested_nodes.retained_heap_charge(work)?)?
            .checked_add(requested_aspects.retained_heap_charge(work)?)?
            .checked_add(admitted_nodes.retained_heap_charge(work)?)?
            .checked_add(admitted_aspects.retained_heap_charge(work)?)?
            .checked_add(skipped_nodes.retained_heap_charge(work)?)?
            .checked_add(skipped_aspects.retained_heap_charge(work)?)?
            .checked_add(no_op_nodes.retained_heap_charge(work)?)?
            .checked_add(no_op_aspects.retained_heap_charge(work)?)?
            .checked_add(support_closure_nodes.retained_heap_charge(work)?)
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
        values.reserve_exact(128);
        ((values.capacity() - before) * std::mem::size_of::<T>()) as u64
    }
    #[test]
    fn replay_scope_charge_tracks_all_vectors_and_owned_digests() {
        let event = emitted_merge_replay_event();
        let mut value = event
            .detail
            .as_ref()
            .unwrap()
            .as_scoped_merge_proof()
            .unwrap()
            .clone();
        let original = value.clone();
        let before = value.retained_heap_charge(&mut Work::new(10000)).unwrap();
        let mut delta = grow_string(&mut value.declaration_digest)
            + grow_string(&mut value.admitted_scope_digest);
        for field in [
            &mut value.skipped_scope_digest,
            &mut value.no_op_scope_digest,
        ]
        .into_iter()
        .flatten()
        {
            delta += grow_string(field);
        }
        delta += grow_vec(&mut value.requested_nodes);
        delta += grow_vec(&mut value.requested_aspects);
        delta += grow_vec(&mut value.admitted_nodes);
        delta += grow_vec(&mut value.admitted_aspects);
        delta += grow_vec(&mut value.skipped_nodes);
        delta += grow_vec(&mut value.skipped_aspects);
        delta += grow_vec(&mut value.no_op_nodes);
        delta += grow_vec(&mut value.no_op_aspects);
        delta += grow_vec(&mut value.support_closure_nodes);
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
