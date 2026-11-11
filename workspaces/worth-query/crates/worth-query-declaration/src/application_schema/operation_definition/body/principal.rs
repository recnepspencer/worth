//! A created actor field binds to admission's principal, never an input member or scope proxy.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreationBuilder,
};
use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationScalarValueBinding, ApplicationSchema,
    },
};

impl<Schema: ApplicationSchema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Copies the exact principal identity field from the admitted observation.
    /// Its codec/value must be the mutation binding's actual identity codec/value.
    /// The selected field and the binding must have the same identity value and codec.
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_operation::ApplicationMutationBinding,
    ///     application_schema::{*, fragment::RecordSelection}};
    /// fn wrong_identity<S: ApplicationSchema, O, I, E, F, B>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>, actor: F,
    /// ) where B: ApplicationMutationBinding<S, Operation=O, Input=I, PrincipalIdentity=String>,
    ///     F: RecordSelection<Entity=E, Value=u64>,
    /// { let _ = creation.initialize_from_principal_identity::<B>(actor); }
    /// ```
    pub fn initialize_from_principal_identity<Binding>(
        mut self,
        field: impl crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Value = Binding::PrincipalIdentity,
            Binding = Binding::PrincipalIdentityBinding,
        >,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
    {
        let field = super::record_authoring::resolve(field)?;
        let principal = Binding::principal_binding();
        if Binding::PrincipalIdentityBinding::UNIT.is_some() {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields
            .push(ApplicationOperationBodyCreatedField::PrincipalIdentity {
                field: field.locus().clone(),
                binding: Binding::PrincipalIdentityBinding::IDENTITY,
                scalar_family: Binding::PrincipalIdentityBinding::SCALAR_FAMILY,
                mutation_binding: Binding::IDENTITY.to_owned(),
                principal_binding: principal.name().to_owned(),
                source: ApplicationFieldBindingLocus::new(
                    principal.principal_entity(),
                    principal.principal_identity_aspect(),
                    principal.principal_identity_field(),
                ),
            });
        Ok(self)
    }
}
