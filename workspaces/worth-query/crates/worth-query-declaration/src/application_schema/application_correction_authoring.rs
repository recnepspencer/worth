//! One typed association installs source, mechanism, disclosure and corrective action.
use super::{ApplicationSchema, ApplicationSchemaDeclarationBuilder};
use crate::application_aftermath::{
    ApplicationCorrectionBinding, ApplicationInverseBinding, DeclaredApplicationAftermathContract,
};
use crate::{
    application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding,
};

impl<Schema: ApplicationSchema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn correction<Correction: ApplicationCorrectionBinding<Schema>>(self) -> Self {
        let definition = Correction::source_definition()
            .aftermath(DeclaredApplicationAftermathContract::correction_binding::<
                Correction,
            >())
            .finish();
        let builder = self.declared_operation(definition);
        let mut builder = if builder
            .member_provenance
            .mutation_bindings()
            .contains(&Correction::Source::descriptor())
        {
            builder
        } else {
            builder.application_mutation_binding::<Correction::Source>()
        };
        for field in Correction::source_output_fields() {
            builder
                .member_provenance
                .register_field_binding(field.binding_recipe().clone());
        }
        if let Some(identity) = Correction::authored_identity() {
            builder
                .member_provenance
                .register_field_binding(identity.binding_recipe().clone());
        }
        let disclosure = Correction::Disclosure::descriptor();
        let builder = if builder
            .member_provenance
            .query_bindings()
            .contains(&disclosure)
        {
            builder
        } else {
            builder.application_query_binding::<Correction::Disclosure>()
        };
        if std::any::TypeId::of::<Correction::Source>()
            == std::any::TypeId::of::<ApplicationInverseBinding<Schema, Correction>>()
            || builder
                .member_provenance
                .mutation_bindings()
                .contains(&ApplicationInverseBinding::<Schema, Correction>::descriptor())
        {
            builder
        } else {
            builder.application_mutation_binding::<ApplicationInverseBinding<Schema, Correction>>()
        }
    }
}
