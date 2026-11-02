//! Resolve a rule's explicit Source requirement against its installed physical plan.
use super::WorthQueryApplicationInvariantSchemaResolver;
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationManagedIndexRef},
    application_schema::ApplicationSchema,
};

impl<S: ApplicationSchema> WorthQueryApplicationInvariantSchemaResolver<'_, S> {
    pub fn typed_managed_index<I: ApplicationManagedIndexBinding<S>>(
        &self, index: ApplicationManagedIndexRef<S, I>,
    ) -> Option<crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantManagedIndexBinding<S, I>>{
        let required = self
            .descriptor
            .managed_index_sources()
            .iter()
            .find(|requirement| requirement.index() == index.name())?;
        if required.definition() != I::descriptor().meaning() {
            return None;
        }
        Some(crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantManagedIndexBinding {
            binding_identity: self.binding_identity.clone(),
            program: self.layout.managed_source(index.name())?.clone(),
            marker: std::marker::PhantomData,
        })
    }
}
