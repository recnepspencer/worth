use std::any::TypeId;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{SemanticSource, WorthQueryApplicationOutputLineage, WorthQueryExactRecordedOutput};

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn qualified_output<Binding: 'static>(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        source_identity: [u8; 32],
    ) -> Option<WorthQueryExactRecordedOutput> {
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope,
            output_binding: TypeId::of::<Binding>(),
        };
        let recorded = self
            .by_source
            .get(&source)?
            .get(&occurrence)?
            .range(..=generation)
            .next_back()
            .map(|(_, recorded)| recorded)?;
        (recorded.source_identity == Some(source_identity)).then_some(())?;
        Some(WorthQueryExactRecordedOutput {
            correspondence: Arc::clone(&recorded.correspondence),
            source_identity,
            source_partition_identity: recorded.source_partition_identity?,
            producer_dependency_identity: recorded.producer_dependency_identity,
            idempotency_key_identity: recorded.idempotency_key_identity,
            runtime_authority: source.runtime_authority,
            schema: source.schema.clone(),
            scope: source.scope,
            observed_source_facts: Arc::clone(&recorded.observed_source_facts),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn has_output_at_or_before<
        Binding: 'static,
    >(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> bool {
        self.by_source.iter().any(|(source, versions)| {
            source.output_binding == TypeId::of::<Binding>()
                && versions
                    .get(&occurrence)
                    .is_some_and(|history| history.range(..=generation).next_back().is_some())
        })
    }
}
