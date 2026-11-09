//! Account relations of the bank model.
use super::*;
pub struct ExternalPrincipal;
impl fragment::Relation for ExternalPrincipal {
    const NAME: &'static str = "ExternalPrincipal";
    type To = Principal;
}
impl ExternalPrincipal {
    pub fn reference(
    ) -> ApplicationRelationRef<BankSchema, Self, ExternalPrincipalMapping, Principal> {
        relation(AccountRecords, ExternalPrincipalMapping, Self)
    }
}
pub struct PrincipalCustomer;
impl fragment::Relation for PrincipalCustomer {
    const NAME: &'static str = "PrincipalCustomer";
    type To = Customer;
}
impl PrincipalCustomer {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, Customer> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct PersonalOwner;
impl fragment::Relation for PersonalOwner {
    const NAME: &'static str = "PersonalOwner";
    type To = Account;
}
impl PersonalOwner {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, Account> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct BusinessOwner;
impl fragment::Relation for BusinessOwner {
    const NAME: &'static str = "BusinessOwner";
    type To = Principal;
}
impl BusinessOwner {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Business, Principal> {
        relation(AccountRecords, Business, Self)
    }
}
pub struct BusinessAccount;
impl fragment::Relation for BusinessAccount {
    const NAME: &'static str = "BusinessAccount";
    type To = Account;
}
impl BusinessAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Business, Account> {
        relation(AccountRecords, Business, Self)
    }
}
pub struct AccountAuthorizedUser;
impl fragment::Relation for AccountAuthorizedUser {
    const NAME: &'static str = "AccountAuthorizedUser";
    type To = AccountAuthorization;
}
impl AccountAuthorizedUser {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, AccountAuthorization>
    {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct AuthorizationAccount;
impl fragment::Relation for AuthorizationAccount {
    const NAME: &'static str = "AuthorizationAccount";
    type To = Account;
}
impl AuthorizationAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, AccountAuthorization, Account> {
        relation(AccountRecords, AccountAuthorization, Self)
    }
}
pub struct InstitutionEmployee;
impl fragment::Relation for InstitutionEmployee {
    const NAME: &'static str = "InstitutionEmployee";
    type To = EmployeeAssignment;
}
impl InstitutionEmployee {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Institution, EmployeeAssignment>
    {
        relation(AccountRecords, Institution, Self)
    }
}
pub struct AssignmentPrincipal;
impl fragment::Relation for AssignmentPrincipal {
    const NAME: &'static str = "AssignmentPrincipal";
    type To = Principal;
}
impl AssignmentPrincipal {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EmployeeAssignment, Principal> {
        relation(AccountRecords, EmployeeAssignment, Self)
    }
}
pub struct InstitutionAccount;
impl fragment::Relation for InstitutionAccount {
    const NAME: &'static str = "InstitutionAccount";
    type To = Account;
}
impl InstitutionAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Institution, Account> {
        relation(AccountRecords, Institution, Self)
    }
}
pub struct InstitutionCashAccount;
impl fragment::Relation for InstitutionCashAccount {
    const NAME: &'static str = "InstitutionCashAccount";
    type To = Account;
}
impl InstitutionCashAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Institution, Account> {
        relation(AccountRecords, Institution, Self)
    }
}
