//! Account records own storage; operation registrations own their existing contracts.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};

pub struct ContractInspectionSchema;
impl Fragment for ContractInspectionSchema {
    const NAME: &'static str = "ContractInspectionSchema";
    type Bindings = Bindings;
    type Members = (
        Account,
        UpdateAccount,
        EmitAccountNotice,
        AccountNoticeEffect,
    );
}
impl ApplicationSchema for ContractInspectionSchema {
    const OWNER: &'static str = "contract_inspection";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountState, AccountAudit);
    type Relations = (ObservedAccount, ChangedAccount);
}
pub struct AccountState;
impl Aspect for AccountState {
    const NAME: &'static str = "AccountState";
    type Row = AccountStateValue;
}
pub struct AccountAudit;
impl Aspect for AccountAudit {
    const NAME: &'static str = "AccountAudit";
    type Row = AccountAuditValue;
}
pub struct AccountStateValue {
    pub status: u64,
    pub balance: u64,
    pub limit: u64,
}
pub struct AccountAuditValue {
    pub sequence: u64,
}
impl Record<AccountStateValue> for Bindings {
    fn record() -> impl Shape<AccountStateValue, Self> {
        field("AccountStatus", |row: &AccountStateValue| row.status)
            .read_only()
            .and("AccountBalance", |row: &AccountStateValue| row.balance)
            .and("AccountLimit", |row: &AccountStateValue| row.limit)
            .read_only()
            .build(|status, balance, limit, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountStateValue {
                    status,
                    balance,
                    limit,
                })
            })
    }
}
impl Record<AccountAuditValue> for Bindings {
    fn record() -> impl Shape<AccountAuditValue, Self> {
        field("AuditSequence", |row: &AccountAuditValue| row.sequence)
            .read_only()
            .build(|sequence, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountAuditValue { sequence })
            })
    }
}
pub fn account() -> ApplicationEntityRef<ContractInspectionSchema, Account> {
    entity(ContractInspectionSchema, Account)
}
pub fn status_field() -> FieldSelection<ContractInspectionSchema, Account, AccountState, u64> {
    select_field(
        ContractInspectionSchema,
        Account,
        AccountState,
        "AccountStatus",
    )
}
pub fn balance_field() -> FieldSelection<ContractInspectionSchema, Account, AccountState, u64> {
    select_field(
        ContractInspectionSchema,
        Account,
        AccountState,
        "AccountBalance",
    )
}
pub fn limit_field() -> FieldSelection<ContractInspectionSchema, Account, AccountState, u64> {
    select_field(
        ContractInspectionSchema,
        Account,
        AccountState,
        "AccountLimit",
    )
}
pub fn sequence_field() -> FieldSelection<ContractInspectionSchema, Account, AccountAudit, u64> {
    select_field(
        ContractInspectionSchema,
        Account,
        AccountAudit,
        "AuditSequence",
    )
}
pub struct ObservedAccount;
impl Relation for ObservedAccount {
    const NAME: &'static str = "ObservedAccount";
    type To = Account;
}
pub struct ChangedAccount;
impl Relation for ChangedAccount {
    const NAME: &'static str = "ChangedAccount";
    type To = Account;
}
pub fn observed_account(
) -> ApplicationRelationRef<ContractInspectionSchema, ObservedAccount, Account, Account> {
    relation(ContractInspectionSchema, Account, ObservedAccount)
}
pub fn changed_account(
) -> ApplicationRelationRef<ContractInspectionSchema, ChangedAccount, Account, Account> {
    relation(ContractInspectionSchema, Account, ChangedAccount)
}
impl Registration<ContractInspectionSchema> for AccountNoticeEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<ContractInspectionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<ContractInspectionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}
impl Registration<ContractInspectionSchema> for UpdateAccount {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<ContractInspectionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<ContractInspectionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 32)
            .operation_read_entity(operation, account())
            .operation_read_field(operation, status_field())
            .operation_read_field(operation, limit_field())
            .operation_read_relation(operation, observed_account())
            .operation_create(operation, account())
            .operation_delete(operation, account())
            .operation_write(operation, balance_field())
            .operation_link(operation, changed_account())
            .operation_unlink(operation, changed_account()))
    }
}
impl Registration<ContractInspectionSchema> for EmitAccountNotice {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<ContractInspectionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<ContractInspectionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect(AccountNoticeEffect::reference(), correlation_family())
                    .aftermath(external_owner_aftermath())
                    .finish(),
            )
            .operation_projection_work_budget(operation, 1)
            .operation_emit(operation, AccountNoticeEffect::reference()))
    }
}
