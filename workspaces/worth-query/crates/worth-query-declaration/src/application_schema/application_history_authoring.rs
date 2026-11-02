use super::{ApplicationSchema, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};
use crate::application_history::{ApplicationHistoryBinding, ApplicationHistoryDefinition};

impl<S: ApplicationSchema> ApplicationSchemaDeclarationBuilder<S> {
    /// Associate already declared ordinary READ and mutation bindings with one model history.
    pub fn application_history<H: ApplicationHistoryBinding<S>>(
        mut self,
        definition: ApplicationHistoryDefinition<S, H>,
    ) -> Self {
        let (definition, descriptor) = definition.into_parts();
        self.member_provenance.register_document_history(descriptor);
        self.push_member(ApplicationSchemaMember::ApplicationHistory { definition })
    }
}
