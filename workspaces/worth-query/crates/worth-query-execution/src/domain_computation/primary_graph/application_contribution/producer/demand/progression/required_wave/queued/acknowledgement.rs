//! Current evidence reaches the sole registry writer by move.
use super::*;
impl QueuedRequiredRoots {
    /// The queued root itself is Current on this basis.
    pub(in super::super) fn acknowledge_root(
        &mut self,
        index: usize,
        proof: performed::CurrentProof<std::sync::Arc<super::WorthQueryOutputDemandSettlement>>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(work) = self.roots[index].take() else {
            return Ok(());
        };
        drop(work.acknowledge_current_admitted(proof)?);
        Ok(())
    }
}
pub(super) fn acknowledge_caller_claim(
    work: SelectedRequiredWork,
    proof: performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>,

) -> Result<
    Option<crate::domain_computation::primary_graph::application_output_demand::ReplacedRequiredWorkHint>,
    WorthQueryOutputDemandDenial,
>{
    work.acknowledge_current_admitted(proof)
}
