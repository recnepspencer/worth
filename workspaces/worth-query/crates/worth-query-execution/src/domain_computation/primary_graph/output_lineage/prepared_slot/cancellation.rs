//! Bounded cleanup of invisible prepared lineage addresses.

use crate::domain_computation::primary_graph::output_lineage::RecordedGeneration;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn drain_cancelled_slots(
        &mut self,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        while let Some(cue) = self.cancelled_slots.as_ref() {
            let coordinate = cue.identity.coordinate();

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
            // Dropping the cue follows unlinking both indexes and the empty cell.
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
