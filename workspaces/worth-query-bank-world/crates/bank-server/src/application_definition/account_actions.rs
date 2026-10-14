//! Exact installed plain actions deliver source completion, without query or workflow authority.
use super::composition::BankAccountsFeature;
use bank_domain::schema::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};
/// Receipt-only completion for the declared CreatePersonalAccount action.
pub struct PersonalAccountCreationDelivery;
impl ApplicationActionDelivery<BankSchema, CreatePersonalAccountMutationBinding>
    for PersonalAccountCreationDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.plain.personalaccountcreation.completion.v1";
}
/// The exact installed CreatePersonalAccount occurrence.
pub type PersonalAccountCreationAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankAccountsFeature,
    CreatePersonalAccountMutationBinding,
    PersonalAccountCreationDelivery,
>;
/// Receipt-only completion for the declared CreateBusinessAccount action.
pub struct BusinessAccountCreationDelivery;
impl ApplicationActionDelivery<BankSchema, CreateBusinessAccountMutationBinding>
    for BusinessAccountCreationDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.plain.businessaccountcreation.completion.v1";
}
/// The exact installed CreateBusinessAccount occurrence.
pub type BusinessAccountCreationAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankAccountsFeature,
    CreateBusinessAccountMutationBinding,
    BusinessAccountCreationDelivery,
>;
/// Receipt-only completion for the declared GrantAccountAuthorization action.
pub struct AccountAccessGrantDelivery;
impl ApplicationActionDelivery<BankSchema, GrantAccountAccessMutationBinding>
    for AccountAccessGrantDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.plain.accountaccessgrant.completion.v1";
}
/// The exact installed GrantAccountAuthorization occurrence.
pub type AccountAccessGrantAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankAccountsFeature,
    GrantAccountAccessMutationBinding,
    AccountAccessGrantDelivery,
>;
/// Receipt-only completion for the declared RevokeAccountAuthorization action.
pub struct AccountAccessRevocationDelivery;
impl ApplicationActionDelivery<BankSchema, RevokeAccountAccessMutationBinding>
    for AccountAccessRevocationDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.plain.accountaccessrevocation.completion.v1";
}
/// The exact installed RevokeAccountAuthorization occurrence.
pub type AccountAccessRevocationAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankAccountsFeature,
    RevokeAccountAccessMutationBinding,
    AccountAccessRevocationDelivery,
>;
