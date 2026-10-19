//! Principal identity is native decision data, never permission supplied by an operand.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperands as Operands,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationScalarValueBinding, ApplicationSchema,
    },
};

impl<Schema: ApplicationSchema, Operation> Operands<Schema, Operation> {
    pub fn principal_identity<Binding>(self, name: &str) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation>,
    {
        let principal = Binding::principal_binding();
        if Binding::PrincipalIdentityBinding::UNIT.is_some() {
            return Err(Denial::UnsupportedScalar);
        }
        self.push(
            name,
            Source::PrincipalIdentity {
                scope: Binding::scope_field().entity().to_owned(),
                mutation_binding: Binding::IDENTITY.to_owned(),
                principal_binding: principal.name().to_owned(),
                field: ApplicationFieldBindingLocus::new(
                    principal.principal_entity(),
                    principal.principal_identity_aspect(),
                    principal.principal_identity_field(),
                ),
                binding: Binding::PrincipalIdentityBinding::IDENTITY,
                scalar_family: Binding::PrincipalIdentityBinding::SCALAR_FAMILY,
            },
        )
    }
}
