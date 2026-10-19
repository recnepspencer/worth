//! A created actor field binds to admission's principal, never an input member or scope proxy.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreationBuilder,
};
use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationScalarValueBinding,
        ApplicationSchema, DeclaredApplicationFieldValue, NoApplicationUnit, OperationWrites,
    },
};

impl<Schema: ApplicationSchema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Copies the exact principal identity field from the admitted observation.
    /// Its codec/value must be the mutation binding's actual identity codec/value.
    /// A binding with another identity value cannot target a `u64` actor field:
    ///
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_operation::ApplicationMutationBinding,
    ///     application_schema::*};
    /// fn wrong_identity<S: ApplicationSchema, O, I, E, A, F, B>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>,
    ///     actor: ApplicationFieldRef<S, E, A, F, u64, ReadOnly, EqualityPredicate, NoApplicationUnit>,
    /// ) where B: ApplicationMutationBinding<S, Operation=O, Input=I, PrincipalIdentity=String>,
    ///     F: OperationWrites<O> + DeclaredApplicationFieldValue<Value=u64>,
    /// {
    ///     let _ = creation.initialize_from_principal_identity::<B, _, _, _, _, _>(actor);
    /// }
    /// ```
    /// Nor can a binding for another operation supply the actor source:
    ///
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_operation::ApplicationMutationBinding,
    ///     application_schema::*};
    /// fn wrong_operation<S: ApplicationSchema, O, Other, I, E, A, F, B>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>,
    ///     actor: ApplicationFieldRef<S, E, A, F, u64, ReadOnly, EqualityPredicate, NoApplicationUnit>,
    /// ) where B: ApplicationMutationBinding<S, Operation=Other, Input=I,
    ///         PrincipalIdentity=u64, PrincipalIdentityBinding=F::Binding>,
    ///     F: OperationWrites<O> + DeclaredApplicationFieldValue<Value=u64>,
    /// {
    ///     let _ = creation.initialize_from_principal_identity::<B, _, _, _, _, _>(actor);
    /// }
    /// ```
    pub fn initialize_from_principal_identity<Binding, Aspect, Field, Value, Write, Equality>(
        mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            NoApplicationUnit,
        >,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
            PrincipalIdentity = Value,
            PrincipalIdentityBinding = Field::Binding,
        >,
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
    {
        let principal = Binding::principal_binding();
        if Field::Binding::UNIT.is_some() {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields
            .push(ApplicationOperationBodyCreatedField::PrincipalIdentity {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
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
