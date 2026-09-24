use super::super::{
    application_contribution::WorthQueryProducerDemandResources, WorthQueryApplicationCommitReceipt,
};
use super::{SemanticSource, WorthQueryApplicationOutputLineage};

impl WorthQueryApplicationOutputLineage {
    /// Only the exact committed output record supplies checkpoint reuse facts.
    /// A source-query footprint or a digest cannot substitute for producer reads.
    pub(in crate::domain_computation::primary_graph) fn producer_facts_for_receipt(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Option<
        std::sync::Arc<[super::super::application_attempt::WorthQueryApplicationObservedFact]>,
    > {
        let output_binding = receipt.output_correspondence().binding_type()?;
        let scope = receipt.principal_scope();
        let idempotency = receipt.idempotency_binding();
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let publication = receipt.committed_product_publication();
        self.by_source
            .get(&source)?
            .get(&publication.product_incarnation())?
            .get(&publication.product_generation().get())?
            .iter()
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
            })?
            .observed_source_facts
            .as_ref()
            .cloned()
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
            output_binding,
        };
        let publication = receipt.committed_product_publication();
        self.by_source
            .get(&source)?
            .get(&publication.product_incarnation())?
            .get(&publication.product_generation().get())?
            .iter()
            .find(|recorded| {
                std::ptr::eq(
                    recorded.correspondence.as_ref(),
                    receipt.output_correspondence(),
                ) && recorded.source_partition_identity
                    == receipt.idempotency_binding().source_partition_identity()
            })?
            .resources
    }

    pub(in crate::domain_computation::primary_graph) fn record_producer_resources(
        &mut self,
        receipt: &WorthQueryApplicationCommitReceipt,
        resources: WorthQueryProducerDemandResources,
    ) -> bool {
        let Some(output_binding) = receipt.output_correspondence().binding_type() else {
            return false;
        };
        let scope = receipt.principal_scope();
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let publication = receipt.committed_product_publication();
        let Some(recorded) = self
            .by_source
            .get_mut(&source)
            .and_then(|occurrences| occurrences.get_mut(&publication.product_incarnation()))
            .and_then(|history| history.get_mut(&publication.product_generation().get()))
            .and_then(|generation| {
                generation.iter_mut().find(|recorded| {
                    std::ptr::eq(
                        recorded.correspondence.as_ref(),
                        receipt.output_correspondence(),
                    ) && recorded.source_partition_identity
                        == receipt.idempotency_binding().source_partition_identity()
                })
            })
        else {
            return false;
        };
        if recorded.resources.is_some_and(|prior| prior != resources) {
            return false;
        }
        recorded.resources = Some(resources);
        true
    }
}
