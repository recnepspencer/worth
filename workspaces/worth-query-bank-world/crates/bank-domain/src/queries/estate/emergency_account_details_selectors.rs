use worth_query_decl::facade::application_query::{
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef, ExactlyOneResult,
    ForwardResultTraversal,
};

use crate::schema::{
    Account, AccountDisplayName, AccountIdentity, BankSchema, EstateAccount, EstateCase, Status,
};

use super::emergency_account_details::EstateEmergencyAccountDetailsQuery;

pub(super) struct AccountRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountRelationSlot => "AccountRelationSlot");
pub(super) struct AccountIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AccountIdentitySlot => "AccountIdentitySlot");
pub(super) struct AccountNameSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountNameSlot => "AccountNameSlot");
pub(super) struct AccountStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(AccountStatusSlot => "AccountStatusSlot");

pub(super) fn estate_account() -> ApplicationQueryResultRelationRef<
    EstateEmergencyAccountDetailsQuery,
    AccountRelationSlot,
    BankSchema,
    EstateAccount,
    EstateCase,
    Account,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("account", EstateAccount::reference())
}

pub(super) fn account_identity() -> ApplicationQueryResultFieldRef<
    EstateEmergencyAccountDetailsQuery,
    AccountIdentitySlot,
    BankSchema,
    AccountIdentity,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::at("AccountIdentity"))
}

pub(super) fn account_name() -> ApplicationQueryResultFieldRef<
    EstateEmergencyAccountDetailsQuery,
    AccountNameSlot,
    BankSchema,
    AccountDisplayName,
> {
    ApplicationQueryResultFieldRef::new(
        "display_name",
        AccountDisplayName::at("AccountDisplayName"),
    )
}

pub(super) fn account_status() -> ApplicationQueryResultFieldRef<
    EstateEmergencyAccountDetailsQuery,
    AccountStatusSlot,
    BankSchema,
    Status,
> {
    ApplicationQueryResultFieldRef::new("status", Status::at("Status"))
}
