//! Moved whole from `domain_computation::primary_graph::output_lineage::semantic_source`: the items below and every impl they own.
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticSource {
    pub(in crate::domain_computation::primary_graph) runtime_authority: u64,
    pub(in crate::domain_computation::primary_graph) schema: ApplicationSchemaBindingIdentity,
    pub scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) output_binding: crate::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity,
}

impl Ord for SemanticSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let prefix = |source: &Self| {
            (
                source.runtime_authority,
                source.schema.runtime_ordinal(),
                source.schema.generation(),
                *source.schema.package_identity(),
                *source.schema.schema_identity(),
                source.scope,
            )
        };
        prefix(self)
            .cmp(&prefix(other))
            .then_with(|| self.output_binding.cmp(&other.output_binding))
    }
}

impl PartialOrd for SemanticSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl SemanticSource {
    #[doc(hidden)]
    pub fn new_for_test(
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        output_binding: crate::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity,
    ) -> Self {
        Self {
            runtime_authority,
            schema,
            scope,
            output_binding,
        }
    }
}

mod court_doors_for_test;
