use super::super::{
    application_contribution::WorthQueryProducerDemandResources, WorthQueryApplicationCommitReceipt,
};
use super::{SemanticSource, WorthQueryApplicationOutputLineage};

impl WorthQueryApplicationOutputLineage {
    /// The exact committed record supplies its producer facts and original
    /// native output witness. Neither a query footprint nor a current head can
    /// substitute for these expectations when capturing reusable output.
    pub(in crate::domain_computation::primary_graph) fn checkpoint_facts_for_receipt(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Option<(
        std::sync::Arc<[super::super::application_attempt::WorthQueryApplicationObservedFact]>,
        &super::SealedNativeOutputWitness,
    )> {
        let output_binding = receipt.output_correspondence().binding_type()?;
        let scope = receipt.principal_scope();
        let idempotency = receipt.idempotency_binding();
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: self.binding_identity(output_binding)?,
        };
        let publication = receipt.committed_product_publication();
        let recorded = self
            .by_source
            .get(&source)?
            .get(&publication.product_incarnation())?
            .get(&publication.product_generation().get())?
            .iter()
            .filter_map(|cell| cell.get())
            .find(|recorded| {
                std::ptr::eq(
                    recorded.correspondence.as_ref(),
                    receipt.output_correspondence(),
                ) && recorded.source_partition_identity == idempotency.source_partition_identity()
                    && recorded.idempotency_key_identity == *idempotency.key_identity()
                    && recorded.producer_dependency_identity
                        == idempotency.producer_dependency_identity()
                    && recorded.source_identity
                        == idempotency.source_identity().map(|identity| {
                            super::RecordedSourceIdentity::Runtime(
                            super::super::application_query::WorthQueryRuntimeSourceIdentity::new(
                                identity,
                            ),
                        )
                        })
            })?;
        Some((
            recorded.checkpoint_source_facts()?,
            recorded.native_output_witness()?,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn producer_resources_for_receipt(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Option<WorthQueryProducerDemandResources> {
        let output_binding = receipt.output_correspondence().binding_type()?;
        let scope = receipt.principal_scope();
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: self.binding_identity(output_binding)?,
        };
        let publication = receipt.committed_product_publication();
        self.by_source
            .get(&source)?
            .get(&publication.product_incarnation())?
            .get(&publication.product_generation().get())?
            .iter()
            .filter_map(|cell| cell.get())
            .find(|recorded| {
                std::ptr::eq(
                    recorded.correspondence.as_ref(),
                    receipt.output_correspondence(),
                ) && recorded.source_partition_identity
                    == receipt.idempotency_binding().source_partition_identity()
            })?
            .resources()
    }
}
