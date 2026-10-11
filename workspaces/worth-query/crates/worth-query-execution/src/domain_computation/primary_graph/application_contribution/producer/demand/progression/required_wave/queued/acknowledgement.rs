//! Current evidence reaches the sole registry writer by move.
use super::*;
impl QueuedRequiredRoots {
    /// The queued root itself is Current on this basis.
    pub(in super::super) fn acknowledge_root(
        &mut self,
        index: usize,
        proof: performed::CurrentProof<std::sync::Arc<super::WorthQueryOutputDemandSettlement>>,
        basis: &performed::CurrentBasis,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let Some(work) = self.roots[index].take() else {
            return Ok(true);
        };
        match work.acknowledge_current_admitted(proof, basis, admission)? {
            crate::domain_computation::primary_graph::application_output_demand::CurrentAcknowledgement::Acknowledged(hint) => { drop(hint); Ok(true) },
            crate::domain_computation::primary_graph::application_output_demand::CurrentAcknowledgement::DifferentBasis(work) => { self.roots[index] = Some(work); Ok(false) },
        }
    }
}
pub(super) fn acknowledge_caller_claim(
    work: SelectedRequiredWork,
    proof: performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>,
    basis: &performed::CurrentBasis,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    crate::domain_computation::primary_graph::application_output_demand::CurrentAcknowledgement,
    WorthQueryOutputDemandDenial,
> {
    work.acknowledge_current_admitted(proof, basis, admission)
}
