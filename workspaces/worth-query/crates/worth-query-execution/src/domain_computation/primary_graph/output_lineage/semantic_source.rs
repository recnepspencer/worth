//! Semantic source order follows the installed binding declaration.
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticSource {
    pub runtime_authority: u64,
    pub schema: ApplicationSchemaBindingIdentity,
    pub scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub output_binding: super::super::output_binding_identity::OutputBindingIdentity,
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
