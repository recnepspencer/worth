//! Field-format verification uses the same sticky, original-funded read session.
use super::*;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::ApplicationContentBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding as Installed;

impl ValueDecodeSession<'_> {
    pub(in crate::domain_computation::primary_graph) fn refuse_content(&self, cause: Stop) {
        self.refuse(cause);
    }
    pub(in crate::domain_computation::primary_graph) fn content_binding<
        C: ApplicationContentBinding,
    >(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<Arc<Installed>, Stop> {
        self.content_binding_matching(entity, aspect, field, Installed::matches_content::<C>)
    }
    fn content_binding_matching(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        matches: fn(&Installed) -> bool,
    ) -> Result<Arc<Installed>, Stop> {
        if let Some(cause) = self.denial() {
            return Err(cause);
        }
        let binding = self.shared_field(entity, aspect, field);
        if let Some(binding) = binding
            && matches(&binding)
        {
            return Ok(binding);
        }
        self.refuse(Stop::BindingContractMismatch);
        Err(match self.denial() {
            Some(cause) => cause,
            None => Stop::Content(
                worth_relational::facade::content::RelationalContentDenial::EncodingInconsistency,
            ),
        })
    }
}
