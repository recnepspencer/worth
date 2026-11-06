mod construction_refusal;
mod declarations;
mod execution;
pub use construction_refusal::BankOperationConstructionRefusal;

pub use declarations::{mutations, BankMutationContract};

use super::BankMutationControls;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};
use bank_domain::schema::BankSchema;
use worth_query_host::facade::declaration::application_schema::{
    ApplicationEncodedScalarValue, ApplicationValueEncodeDenial, OperationExpectsFact,
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
    ///         .expect_version(Status::at("Status"), AccountStatus::Open);
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
    ///             AccountingRevision::at("AccountingRevision"),
    ///             AccountJournalRevision::default(),
    ///         );
    /// }
    /// ```
    pub fn expect_version<Aspect, Field, Value>(
        mut self,
        field: Field,
        expected: Value,
    ) -> Result<Self, BankMutationPreconditionDenial>
    where
        Field: OperationExpectsVersion<Operation>
            + RecordSelection<Entity = Scope, Aspect = Aspect, Value = Value>,
    {
        let expected = ApplicationEncodedScalarValue::<Field::Binding>::try_new(expected)
            .map_err(BankMutationPreconditionDenial::Encoding)?;
        self.preconditions = self
            .preconditions
            .expect_version(field, expected)
            .map_err(|denial| BankMutationPreconditionDenial::Selection(Box::new(denial)))?;
        Ok(self)
    }

    pub fn expect_fact<Aspect, Field, Value>(
        mut self,
        field: Field,
        expected: Value,
    ) -> Result<Self, BankMutationPreconditionDenial>
    where
        Field: OperationExpectsFact<Operation>
            + RecordSelection<Entity = Scope, Aspect = Aspect, Value = Value>,
    {
        let expected = ApplicationEncodedScalarValue::<Field::Binding>::try_new(expected)
            .map_err(BankMutationPreconditionDenial::Encoding)?;
        self.preconditions = self
            .preconditions
            .expect_fact(field, expected)
            .map_err(|denial| BankMutationPreconditionDenial::Selection(Box::new(denial)))?;
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

use worth_query_host::facade::declaration::application_schema::fragment::RecordSelection;
#[derive(Debug)]
pub enum BankMutationPreconditionDenial {
    Encoding(ApplicationValueEncodeDenial),
    Selection(
        Box<worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationDenial>,
    ),
}
impl std::fmt::Display for BankMutationPreconditionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mutation precondition refused: {self:?}")
    }
}
impl std::error::Error for BankMutationPreconditionDenial {}
