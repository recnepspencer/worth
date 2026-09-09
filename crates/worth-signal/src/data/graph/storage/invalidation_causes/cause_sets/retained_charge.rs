use super::CanonicalCauseSetStore;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for CanonicalCauseSetStore {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            generation: _,
            sets,
            slot_generations,
            free_indices,
            next_output_commit_ordinal: _,
            published_output_commits,
            occupied_set_count: _,
            output_commit_reference_counts,
            retained_custody,
            deserialized_quarantine: _,
            #[cfg(test)]
                published_order_probe: _,
            #[cfg(test)]
                last_compaction_slot_visits: _,
        } = self;
        let charge = sets
            .retained_heap_charge(work)?
            .checked_add(slot_generations.retained_heap_charge(work)?)?
            .checked_add(free_indices.retained_heap_charge(work)?)?
            .checked_add(published_output_commits.retained_heap_charge(work)?)?
            .checked_add(output_commit_reference_counts.retained_heap_charge(work)?)?
            .checked_add(retained_custody.retained_heap_charge(work)?)?;
        Ok(charge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::Aspect;
    use crate::data::handle::NodeId;
    use crate::data::output::PartitionSubscription;
    use crate::data::proof::invalidation::binding::{
        DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
    };
    use crate::data::proof::PartitionScopeSet;

    #[test]
    fn released_fork_cause_still_charges_hidden_base_scopes_and_denies_partial_preparation() {
        let name = "partition".repeat(2_048);
        let minimum_payload = name.len() as u64 * 3;
        let scope = PartitionSubscription::whole_partition(name);
        let cause = ResolvedDependencyCause::new(
            1,
            NodeId::new(1, 0),
            DependencyRevision(1),
            NodeId::new(2, 0),
            Aspect::new(0),
            Some(scope.clone()),
            0,
            OutputCommitOrdinal(1),
            1,
            PartitionScopeSet::new([scope]),
        );
        let mut parent = CanonicalCauseSetStore::default();
        let id = parent.insert([cause]);
        let mut retained = parent.fork_persistent();
        retained.release(id).unwrap();
        drop(parent);
        let mut work = Preparation::new(1_000);
        let charge = retained.retained_heap_charge(&mut work).unwrap();
        assert!(
            charge.bytes() >= minimum_payload,
            "released selected cause retains all three base scope representations"
        );
        let required = work.visits();
        assert_eq!(
            retained
                .retained_heap_charge(&mut Preparation::new(required))
                .unwrap(),
            charge
        );
        assert!(matches!(
            retained.retained_heap_charge(&mut Preparation::new(required - 1)),
            Err(Denial::WorkExhausted { .. })
        ));
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for CanonicalCauseSetStore {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let Self {
            generation: _,
            sets,
            slot_generations,
            free_indices,
            next_output_commit_ordinal: _,
            published_output_commits,
            occupied_set_count: _,
            output_commit_reference_counts,
            retained_custody,
            deserialized_quarantine: _,
            #[cfg(test)]
                published_order_probe: _,
            #[cfg(test)]
                last_compaction_slot_visits: _,
        } = self;
        let charge = sets
            .prepare_fork_charge(work)?
            .checked_add(slot_generations.prepare_fork_charge(work)?)?
            .checked_add(free_indices.prepare_fork_charge(work)?)?
            .checked_add(published_output_commits.prepare_fork_charge(work)?)?
            .checked_add(output_commit_reference_counts.prepare_fork_charge(work)?)?
            .checked_add(RetainedStorageForkCharge::unchanged(
                retained_custody.retained_heap_charge(work)?,
            ))?;
        Ok(charge)
    }
}
