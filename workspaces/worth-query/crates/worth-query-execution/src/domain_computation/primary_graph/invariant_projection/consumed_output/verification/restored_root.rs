//! A restored output verified by its first reader.
//!
//! Installation records a checkpoint output without verifying it, so the row
//! carries no output witness until a demand or a reader compares it. The
//! reader builds the witness from the checkpoint facts and compares those
//! facts and the output in full at its own snapshot, under the same rule a
//! demand readmits the output by.

use super::*;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence;

impl ConsumedOutputEvidence {
    /// The witness of a restored output whose facts and output are unchanged
    /// at `snapshot`. `None` when the facts do not cover the output or either
    /// has changed.
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn verify_restored_root_at(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        layout: &WorthQueryPrimaryGraphLayout,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<Option<Arc<OnceLock<SealedNativeOutputWitness>>>, ConsumedOutputVerificationStop>
    {
        SealedNativeOutputWitness::from_checkpoint_facts(correspondence, layout, source_facts)
            .map_err(map_admission_stop)
            .and_then(|witness| {
                let Some(witness) = witness else {
                    return Ok(None);
                };
                let current = witness
                    .get()
                    .expect("checkpoint constructor sealed its witness")
                    .checkpoint_facts_current_in(execution, runtime, snapshot, source_facts)
                    .map_err(map_verification_stop)?;
                Ok(current.then_some(witness))
            })
    }
}
