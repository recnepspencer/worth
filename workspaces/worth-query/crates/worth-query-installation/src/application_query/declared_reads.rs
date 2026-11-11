//! Descriptive dependency scope comes from the exact installed Query binding.
use super::WorthQueryInstalledGraphReadContract;
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use worth_query_declaration::facade::application_query::ApplicationQueryBindingReference;
impl<S> WorthQueryInstalledApplicationSchema<S> {
    /// Does not admit a read. A declared dependency is returned only for the
    /// matching installed descriptor, including its source and schema meaning.
    pub fn declared_query_read_contract(
        &self,
        reference: &ApplicationQueryBindingReference,
    ) -> Option<WorthQueryInstalledGraphReadContract> {
        let binding = self
            .query_catalog
            .get_binding(reference.parts().binding.as_str())?;
        reference
            .matches_descriptor(binding.descriptor())
            .then(|| binding.query().read_graph().clone())
    }
}
