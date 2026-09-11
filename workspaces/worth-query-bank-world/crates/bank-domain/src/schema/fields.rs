use worth_query_decl::facade::{worth_query_aspect, worth_query_field};

use crate::model::{
    AccountAuthorizationId, AccountId, AccountJournalRevision, AccountName, BusinessId,
    CustomerRole, EmployeeAssignmentId, EmployeeRole, InstitutionId, JournalEntryId, Money,
    PaymentId, PostingId, SignedMoney, USD,
};

use super::entities::{
    Account, AccountAuthorization, Business, EmployeeAssignment, Institution, JournalEntry,
    PaymentIntent, Posting,
};
use super::governance::UsdCurrency;
use super::values::{
    AccountAuthorizationIdBinding, AccountIdBinding, AccountJournalRevisionBinding, AccountKind,
    AccountKindBinding, AccountNameBinding, AccountStatus, AccountStatusBinding, BusinessIdBinding,
    CustomerRoleBinding, EmployeeAssignmentIdBinding, EmployeeRoleBinding, InstitutionIdBinding,
    JournalEntryIdBinding, PaymentIdBinding, PaymentStatus, PaymentStatusBinding, PostingIdBinding,
    PostingPurpose, PostingPurposeBinding, SignedUsdMoneyBinding, UsdMoneyBinding,
};
use super::BankSchema;

worth_query_aspect!(pub Identity for BankSchema, Account; identity = AspectIdentity(0x9161100a), revision = AspectContractRevision(1),);
worth_query_aspect!(pub InstitutionIdentity for BankSchema, Institution; identity = AspectIdentity(0x9161100b), revision = AspectContractRevision(1),);
worth_query_aspect!(pub BusinessIdentity for BankSchema, Business; identity = AspectIdentity(0x9161100c), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PaymentIdentity for BankSchema, PaymentIntent; identity = AspectIdentity(0x9161100d), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AccountProfile for BankSchema, Account; identity = AspectIdentity(0x9161100e), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AccountState for BankSchema, Account; identity = AspectIdentity(0x9161100f), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AuthorizationScope for BankSchema, AccountAuthorization; identity = AspectIdentity(0x91611010), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AuthorizationIdentity for BankSchema, AccountAuthorization; identity = AspectIdentity(0x91611011), revision = AspectContractRevision(1),);
worth_query_aspect!(pub EmployeeScope for BankSchema, EmployeeAssignment; identity = AspectIdentity(0x91611012), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PostingValue for BankSchema, Posting; identity = AspectIdentity(0x91611013), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PostingIdentity for BankSchema, Posting; identity = AspectIdentity(0x91611014), revision = AspectContractRevision(1),);
worth_query_aspect!(pub JournalIdentity for BankSchema, JournalEntry; identity = AspectIdentity(0x91611015), revision = AspectContractRevision(1),);
worth_query_aspect!(pub JournalState for BankSchema, JournalEntry; identity = AspectIdentity(0x91611016), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PaymentState for BankSchema, PaymentIntent; identity = AspectIdentity(0x91611017), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PaymentValue for BankSchema, PaymentIntent; identity = AspectIdentity(0x91611018), revision = AspectContractRevision(1),);

worth_query_field!(
    pub AccountIdentity for BankSchema, Account, Identity:
    AccountId => AccountIdBinding, read_only, equality
);
worth_query_field!(
    pub InstitutionIdentityField for BankSchema, Institution, InstitutionIdentity:
    InstitutionId => InstitutionIdBinding, read_only, equality
);
worth_query_field!(
    pub BusinessIdentityField for BankSchema, Business, BusinessIdentity:
    BusinessId => BusinessIdBinding, read_only, equality
);
worth_query_field!(
    pub PaymentIdentityField for BankSchema, PaymentIntent, PaymentIdentity:
    PaymentId => PaymentIdBinding, read_only, equality
);
worth_query_field!(
    pub AccountDisplayName for BankSchema, Account, AccountProfile:
    AccountName => AccountNameBinding, read_write, equality
);
worth_query_field!(
    pub Kind for BankSchema, Account, AccountProfile:
    AccountKind => AccountKindBinding, read_write, equality
);
worth_query_field!(
    pub AccountingRevision for BankSchema, Account, AccountState:
    AccountJournalRevision => AccountJournalRevisionBinding, read_write, equality
);
worth_query_field!(
    pub Status for BankSchema, Account, AccountState:
    AccountStatus => AccountStatusBinding, read_write, equality
);
worth_query_field!(
    pub AuthorizationRole for BankSchema, AccountAuthorization, AuthorizationScope:
    CustomerRole => CustomerRoleBinding, read_write, equality
);
worth_query_field!(
    pub AccountAuthorizationIdentity for BankSchema, AccountAuthorization, AuthorizationIdentity:
    AccountAuthorizationId => AccountAuthorizationIdBinding, read_only, equality
);
worth_query_field!(
    pub EmployeeAssignmentIdentityField for BankSchema, EmployeeAssignment, EmployeeScope:
    EmployeeAssignmentId => EmployeeAssignmentIdBinding, read_only, equality
);
worth_query_field!(
    pub AssignmentRole for BankSchema, EmployeeAssignment, EmployeeScope:
    EmployeeRole => EmployeeRoleBinding, read_write, equality
);
worth_query_field!(
    pub PostingAmount for BankSchema, Posting, PostingValue:
    SignedMoney<USD> => SignedUsdMoneyBinding, unit UsdCurrency, read_write, no_equality
);
worth_query_field!(
    pub PostingAccountSequence for BankSchema, Posting, PostingValue:
    AccountJournalRevision => AccountJournalRevisionBinding, read_write, equality
);
worth_query_field!(
    pub PostingIdentityField for BankSchema, Posting, PostingIdentity:
    PostingId => PostingIdBinding, read_only, equality
);
worth_query_field!(
    pub Purpose for BankSchema, Posting, PostingValue:
    PostingPurpose => PostingPurposeBinding, read_write, equality
);
worth_query_field!(
    pub PaymentStatusField for BankSchema, PaymentIntent, PaymentState:
    PaymentStatus => PaymentStatusBinding, read_write, equality
);
worth_query_field!(
    pub JournalIdentityField for BankSchema, JournalEntry, JournalIdentity:
    JournalEntryId => JournalEntryIdBinding, read_only, equality
);
worth_query_field!(
    pub JournalPurpose for BankSchema, JournalEntry, JournalState:
    PostingPurpose => PostingPurposeBinding, read_write, equality
);
worth_query_field!(
    pub PaymentAmount for BankSchema, PaymentIntent, PaymentValue:
    Money<USD> => UsdMoneyBinding, unit UsdCurrency, read_write, no_equality
);
