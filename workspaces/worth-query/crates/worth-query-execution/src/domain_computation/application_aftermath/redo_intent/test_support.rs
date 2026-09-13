use super::*;

impl WorthQueryProvedUndo {
    pub(crate) fn axis_probe(parts: WorthQueryProvedUndoAxisProbe) -> Self {
        let publication =
            crate::domain_computation::primary_graph::committed_recoverable_application()
                .committed_product_publication()
                .clone();
        Self::axis_probe_with_product_publication(parts, publication)
    }

    pub(crate) fn axis_probe_with_product_publication(
        parts: WorthQueryProvedUndoAxisProbe,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> Self {
        Self::axis_probe_with_parts(parts, publication.relational_commit().clone(), publication)
    }

    fn axis_probe_with_parts(
        parts: WorthQueryProvedUndoAxisProbe,
        undo_commit: RelationalCommitReceipt,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> Self {
        let performed =
            Performed::<WorthQueryCompletedUndoAction, WorthQueryUndoCompletionAuthority>::record(
                &WorthQueryUndoCompletionAuthority::witness(),
                (),
            );
        Self {
            _completion: Proof::from_authority_witness(
                &WorthQueryUndoCompletionAuthority::witness(),
            ),
            causal: prove_inversion(&performed),
            original_operation: parts.original_operation,
            undo_commit,
            undo_product_publication: publication,
            principal_scope_digest: parts.principal_scope_digest,
            compatibility_generation: parts.compatibility_generation,
            runtime_instance: parts.runtime_instance,
            _private: (),
        }
    }
}
