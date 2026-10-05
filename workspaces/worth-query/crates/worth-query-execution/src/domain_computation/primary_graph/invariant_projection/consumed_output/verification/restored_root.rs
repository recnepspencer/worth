//! A restored output verified by its first reader.
//!
//! Installation records a checkpoint output without verifying it, so the row
//! carries no output witness until a demand or a reader compares it. The
//! reader builds the witness from the checkpoint facts and compares those
//! facts and the output in full at its own snapshot, under the same rule a
//! demand readmits the output by.

use super::*;
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryApplicationOutputCorrespondence,
};

impl ConsumedOutputEvidence {
    /// The witness of a restored output whose facts and output are unchanged
    /// at `snapshot`. `None` when the facts do not cover the output or either
    /// has changed.
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn verify_restored_root_at(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        source_facts: &[WorthQueryApplicationObservedFact],
        layout: &WorthQueryPrimaryGraphLayout,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        remaining_work: &mut usize,
    ) -> Result<Option<Arc<OnceLock<SealedNativeOutputWitness>>>, ConsumedOutputVerificationStop>
    {
        let mut admission = owner.read_admission(*remaining_work);
        let witness = SealedNativeOutputWitness::from_checkpoint_facts(
            correspondence,
            layout,
            source_facts,
            owner,
            &mut admission,
        )
        .and_then(|witness| {
            let Some(witness) = witness else {
                return Ok(None);
            };
            let current = witness
                .get()
                .expect("checkpoint constructor sealed its witness")
                .checkpoint_facts_current_in(runtime, snapshot, source_facts, &mut admission)?;
            Ok(current.then_some(witness))
        })
        .map_err(map_admission_stop);
        debit_wrapper_work(&admission, remaining_work)?;
        witness
    }
}
