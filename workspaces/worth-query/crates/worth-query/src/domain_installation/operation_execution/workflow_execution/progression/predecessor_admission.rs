use crate::basis_lifecycle::BasisOperationLane;

use super::{
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind, WorthQueryWorkflowRun,
    WorthQueryWorkflowStageReceipt,
};

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(super) fn predecessor_receipt_indices(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Result<Vec<usize>, WorthQueryWorkflowAdvanceDenial> {
        stage
            .predecessors()
            .iter()
            .map(|predecessor| {
                self.counters.predecessor_receipt_lookups += 1;
                self.receipt_index
                    .get(predecessor)
                    .copied()
                    .filter(|index| self.receipts.get(*index).is_some())
                    .ok_or_else(|| {
                        WorthQueryWorkflowAdvanceDenial::new(
                            WorthQueryWorkflowAdvanceDenialKind::PredecessorAuthorityMissing(
                                predecessor.clone(),
                            ),
                            self.counters,
                        )
                    })
            })
            .collect()
    }

    pub(super) fn assert_predecessor_authority(
        &self,
        receipts: &[&WorthQueryWorkflowStageReceipt],
    ) {
        debug_assert!(receipts.iter().all(|receipt| {
            std::sync::Arc::ptr_eq(&receipt.authority_proof, &self.authority_proof)
                && std::sync::Arc::ptr_eq(
                    &receipt.stage_authority_proof.run_authority,
                    &self.authority_proof,
                )
                && receipt.stage_authority_proof.proof.payload().identity() == receipt.identity
                && receipt.stage_authority_proof.stage_identity == receipt.stage_identity
                && receipt.authority_proof.binding_identity() == self.bound.binding_identity()
                && receipt.authority_proof.capability_identity() == self.bound.capability_identity()
        }));
    }
}
