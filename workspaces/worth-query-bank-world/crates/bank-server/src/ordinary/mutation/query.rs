mod declarations;
mod execution;

pub use declarations::{mutations, BankMutationContract};

use super::BankMutationControls;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};
use bank_domain::schema::BankSchema;
use worth_query_host::facade::declaration::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationValueEncodeDenial, DeclaredApplicationFieldValue, OperationExpectsFact,
    OperationExpectsVersion, TypedMutationPreconditions,
};

pub struct BankMutation<'runtime, Mutation> {
    runtime: &'runtime BankIdentityRuntime,
    mutation: Mutation,
}

pub struct BankMutationForPrincipal<'runtime, 'principal, Mutation, Operation, Scope> {
    runtime: &'runtime BankIdentityRuntime,
    mutation: Mutation,
    principal: &'principal BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<BankSchema, Operation, Scope>,
}

pub struct BankReadyMutation<'runtime, 'principal, Mutation, Operation, Scope> {
    runtime: &'runtime BankIdentityRuntime,
    mutation: Mutation,
    principal: &'principal BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<BankSchema, Operation, Scope>,
    controls: BankMutationControls,
}

impl BankIdentityRuntime {
    pub const fn mutate<Mutation>(&self, mutation: Mutation) -> BankMutation<'_, Mutation> {
        BankMutation {
            runtime: self,
            mutation,
        }
    }
}

impl<'runtime, Mutation> BankMutation<'runtime, Mutation> {
    pub fn as_principal<'principal>(
        self,
        principal: &'principal BankAuthenticatedPrincipal,
    ) -> BankMutationForPrincipal<
        'runtime,
        'principal,
        Mutation,
        Mutation::Operation,
        Mutation::Scope,
    >
    where
        Mutation: BankMutationContract,
    {
        BankMutationForPrincipal {
            runtime: self.runtime,
            mutation: self.mutation,
            principal,
            preconditions: TypedMutationPreconditions::new(),
        }
    }
}

impl<'runtime, 'principal, Mutation, Operation, Scope>
    BankMutationForPrincipal<'runtime, 'principal, Mutation, Operation, Scope>
{
    /// Adds a typed expected-version comparison declared for this operation.
    ///
    /// Wrong precondition families and fields declared for another operation
    /// are rejected by the compiler in one consolidated certification target:
    ///
    /// ```compile_fail,E0277
    /// use bank_domain::schema::{AccountStatus, SendMoney, Status};
    /// use bank_server::{
    ///     mutations, BankAuthenticatedPrincipal, BankIdentityRuntime,
    /// };
    ///
    /// fn wrong_family(
    ///     runtime: &BankIdentityRuntime,
    ///     principal: &BankAuthenticatedPrincipal,
    ///     input: SendMoney,
    /// ) {
    ///     runtime
    ///         .mutate(mutations::send_money(input))
    ///         .as_principal(principal)
    ///         .expect_version(Status::reference(), AccountStatus::Open);
    /// }
    /// ```
    ///
    /// ```compile_fail,E0277
    /// use bank_domain::model::AccountJournalRevision;
    /// use bank_domain::schema::{AccountingRevision, Deposit};
    /// use bank_server::{
    ///     mutations, BankAuthenticatedPrincipal, BankIdentityRuntime,
    /// };
    ///
    /// fn wrong_operation(
    ///     runtime: &BankIdentityRuntime,
    ///     principal: &BankAuthenticatedPrincipal,
    ///     input: Deposit,
    /// ) {
    ///     runtime
    ///         .mutate(mutations::deposit(input))
    ///         .as_principal(principal)
    ///         .expect_version(
    ///             AccountingRevision::reference(),
    ///             AccountJournalRevision::default(),
    ///         );
    /// }
    /// ```
    pub fn expect_version<Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<BankSchema, Scope, Aspect, Field, Value, Write, Equality, Unit>,
        expected: Value,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Field: OperationExpectsVersion<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        let expected = ApplicationEncodedScalarValue::<Field::Binding>::try_new(expected)?;
        self.preconditions = self.preconditions.expect_version(field, expected);
        Ok(self)
    }

    pub fn expect_fact<Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<BankSchema, Scope, Aspect, Field, Value, Write, Equality, Unit>,
        expected: Value,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Field: OperationExpectsFact<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        let expected = ApplicationEncodedScalarValue::<Field::Binding>::try_new(expected)?;
        self.preconditions = self.preconditions.expect_fact(field, expected);
        Ok(self)
    }

    pub fn controls(
        self,
        controls: BankMutationControls,
    ) -> BankReadyMutation<'runtime, 'principal, Mutation, Operation, Scope> {
        BankReadyMutation {
            runtime: self.runtime,
            mutation: self.mutation,
            principal: self.principal,
            preconditions: self.preconditions,
            controls,
        }
    }
}
