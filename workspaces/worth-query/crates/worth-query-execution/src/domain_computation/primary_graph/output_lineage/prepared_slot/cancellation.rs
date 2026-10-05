//! Bounded cleanup of invisible prepared lineage addresses.

use super::{denial, tree_work};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, RecordedGeneration, SemanticSource,
    WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};
use worth_runtime_world::facade::ProductBranchIncarnation;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn drain_cancelled_slots(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        while let Some(cue) = self.cancelled_slots.as_ref() {
            let source = cue.identity.source();
            let coordinate = cue.identity.coordinate();
            let outer = tree_work::<SemanticSource>(self.by_source.len())
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
            admission
                .charge_external_work(outer)
                .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
            let occurrences = self
                .by_source
                .get(source)
                .expect("cancelled source remains retained");
            let middle = tree_work::<ProductBranchIncarnation>(occurrences.len())
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
            admission
                .charge_external_work(middle)
                .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
            let history = occurrences
                .get(&coordinate.occurrence)
                .expect("cancelled occurrence remains retained");
            let inner =
                tree_work::<u64>(history.len()).ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
            // Removal can delete the final source and occurrence as well as
            // the generation. Reserve each selected path before unlinking.
            admission
                .charge_external_work(
                    inner
                        .checked_mul(2)
                        .and_then(|work| work.checked_add(middle))
                        .and_then(|work| work.checked_add(outer))
                        .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
                )
                .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
            self.partition_index.prepare_cancelled_removal(
                source,
                coordinate,
                cue.partition,
                admission,
            )?;
            let mut cue = self.cancelled_slots.take().unwrap();
            self.cancelled_slots = cue.next.take();
            self.partition_index.remove_vacancy(
                cue.identity.source(),
                coordinate.occurrence,
                coordinate.generation,
                cue.partition,
            );
            let occurrences = self.by_source.get_mut(cue.identity.source()).unwrap();
            let history = occurrences.get_mut(&coordinate.occurrence).unwrap();
            let removed: RecordedGeneration = history
                .remove(&coordinate.generation)
                .expect("cancelled generation remains retained");
            assert!(removed.iter().all(|cell| cell.get().is_none()));
            if history.is_empty() {
                occurrences.remove(&coordinate.occurrence);
            }
            if occurrences.is_empty() {
                self.by_source.remove(cue.identity.source());
            }
            // The cue owns the retained capacity until both derived indexes
            // and the empty record cell are gone.
        }
        Ok(())
    }
}

impl Drop for WorthQueryApplicationOutputLineage {
    fn drop(&mut self) {
        let mut cue = self.cancelled_slots.take();
        while let Some(mut current) = cue {
            cue = current.next.take();
        }
    }
}
