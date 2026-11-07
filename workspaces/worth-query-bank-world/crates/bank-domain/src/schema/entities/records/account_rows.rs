//! Account rows of the bank model.
use super::*;
pub struct Identity;
impl fragment::Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = IdentityValue;
}
impl Identity {
    pub fn reference() -> AspectSelection<AccountRecords, Account, Self> {
        aspect(AccountRecords, Account, Self)
    }
}
pub struct IdentityValue {
    pub account_identity: AccountId,
}
impl fragment::Record<IdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<IdentityValue, Self> {
        fragment::field("AccountIdentity", |row: &IdentityValue| {
            row.account_identity
        })
        .equality_queryable(true)
        .build(|account_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(IdentityValue { account_identity })
        })
    }
}
pub struct InstitutionIdentity;
impl fragment::Aspect for InstitutionIdentity {
    const NAME: &'static str = "InstitutionIdentity";
    type Row = InstitutionIdentityValue;
}
impl InstitutionIdentity {
    pub fn reference() -> AspectSelection<AccountRecords, Institution, Self> {
        aspect(AccountRecords, Institution, Self)
    }
}
pub struct InstitutionIdentityValue {
    pub institution_identity_field: InstitutionId,
}
impl fragment::Record<InstitutionIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<InstitutionIdentityValue, Self> {
        fragment::field(
            "InstitutionIdentityField",
            |row: &InstitutionIdentityValue| row.institution_identity_field,
        )
        .equality_queryable(true)
        .build(
            |institution_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(InstitutionIdentityValue {
                    institution_identity_field,
                })
            },
        )
    }
}
pub struct BusinessIdentity;
impl fragment::Aspect for BusinessIdentity {
    const NAME: &'static str = "BusinessIdentity";
    type Row = BusinessIdentityValue;
}
impl BusinessIdentity {
    pub fn reference() -> AspectSelection<AccountRecords, Business, Self> {
        aspect(AccountRecords, Business, Self)
    }
}
pub struct BusinessIdentityValue {
    pub business_identity_field: BusinessId,
}
impl fragment::Record<BusinessIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<BusinessIdentityValue, Self> {
        fragment::field("BusinessIdentityField", |row: &BusinessIdentityValue| {
            row.business_identity_field
        })
        .equality_queryable(true)
        .build(
            |business_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(BusinessIdentityValue {
                    business_identity_field,
                })
            },
        )
    }
}
pub struct AccountProfile;
impl fragment::Aspect for AccountProfile {
    const NAME: &'static str = "AccountProfile";
    type Row = AccountProfileValue;
}
impl AccountProfile {
    pub fn reference() -> AspectSelection<AccountRecords, Account, Self> {
        aspect(AccountRecords, Account, Self)
    }
}
pub struct AccountProfileValue {
    pub account_display_name: AccountName,
    pub kind: AccountKind,
}
impl fragment::Record<AccountProfileValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<AccountProfileValue, Self> {
        fragment::field("AccountDisplayName", |row: &AccountProfileValue| {
            row.account_display_name.clone()
        })
        .equality_queryable(true)
        .and("Kind", |row: &AccountProfileValue| row.kind)
        .equality_queryable(true)
        .build(
            |account_display_name, kind, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountProfileValue {
                    account_display_name,
                    kind,
                })
            },
        )
    }
}
pub struct AccountState;
impl fragment::Aspect for AccountState {
    const NAME: &'static str = "AccountState";
    type Row = AccountStateValue;
}
impl AccountState {
    pub fn reference() -> AspectSelection<AccountRecords, Account, Self> {
        aspect(AccountRecords, Account, Self)
    }
}
pub struct AccountStateValue {
    pub accounting_revision: AccountJournalRevision,
    pub status: AccountStatus,
}
impl fragment::Record<AccountStateValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<AccountStateValue, Self> {
        fragment::field("AccountingRevision", |row: &AccountStateValue| {
            row.accounting_revision
        })
        .equality_queryable(true)
        .and("Status", |row: &AccountStateValue| row.status)
        .equality_queryable(true)
        .build(
            |accounting_revision, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountStateValue {
                    accounting_revision,
                    status,
                })
            },
        )
    }
}
pub struct AuthorizationScope;
impl fragment::Aspect for AuthorizationScope {
    const NAME: &'static str = "AuthorizationScope";
    type Row = AuthorizationScopeValue;
}
impl AuthorizationScope {
    pub fn reference() -> AspectSelection<AccountRecords, AccountAuthorization, Self> {
        aspect(AccountRecords, AccountAuthorization, Self)
    }
}
pub struct AuthorizationScopeValue {
    pub authorization_role: CustomerRole,
}
impl fragment::Record<AuthorizationScopeValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<AuthorizationScopeValue, Self> {
        fragment::field("AuthorizationRole", |row: &AuthorizationScopeValue| {
            row.authorization_role
        })
        .equality_queryable(true)
        .build(|authorization_role, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AuthorizationScopeValue { authorization_role })
        })
    }
}
pub struct AuthorizationIdentity;
impl fragment::Aspect for AuthorizationIdentity {
    const NAME: &'static str = "AuthorizationIdentity";
    type Row = AuthorizationIdentityValue;
}
impl AuthorizationIdentity {
    pub fn reference() -> AspectSelection<AccountRecords, AccountAuthorization, Self> {
        aspect(AccountRecords, AccountAuthorization, Self)
    }
}
pub struct AuthorizationIdentityValue {
    pub account_authorization_identity: AccountAuthorizationId,
}
impl fragment::Record<AuthorizationIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<AuthorizationIdentityValue, Self> {
        fragment::field(
            "AccountAuthorizationIdentity",
            |row: &AuthorizationIdentityValue| row.account_authorization_identity,
        )
        .equality_queryable(true)
        .build(
            |account_authorization_identity, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AuthorizationIdentityValue {
                    account_authorization_identity,
                })
            },
        )
    }
}
pub struct EmployeeScope;
impl fragment::Aspect for EmployeeScope {
    const NAME: &'static str = "EmployeeScope";
    type Row = EmployeeScopeValue;
}
impl EmployeeScope {
    pub fn reference() -> AspectSelection<AccountRecords, EmployeeAssignment, Self> {
        aspect(AccountRecords, EmployeeAssignment, Self)
    }
}
pub struct EmployeeScopeValue {
    pub employee_assignment_identity_field: EmployeeAssignmentId,
    pub assignment_role: EmployeeRole,
}
impl fragment::Record<EmployeeScopeValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<EmployeeScopeValue, Self> {
        fragment::field(
            "EmployeeAssignmentIdentityField",
            |row: &EmployeeScopeValue| row.employee_assignment_identity_field,
        )
        .equality_queryable(true)
        .and("AssignmentRole", |row: &EmployeeScopeValue| {
            row.assignment_role
        })
        .equality_queryable(true)
        .build(
            |employee_assignment_identity_field,
             assignment_role,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EmployeeScopeValue {
                    employee_assignment_identity_field,
                    assignment_role,
                })
            },
        )
    }
}
