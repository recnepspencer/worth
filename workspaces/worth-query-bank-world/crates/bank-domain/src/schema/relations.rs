use worth_query_decl::facade::worth_query_relation;

use super::entities::{
    Account, AccountAuthorization, Approval, Business, Customer, EmployeeAssignment,
    ExternalPrincipalMapping, Institution, JournalEntry, PaymentIntent, Posting, Principal,
};
use super::BankSchema;

worth_query_relation!(pub ExternalPrincipal in BankSchema, ExternalPrincipalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PrincipalCustomer in BankSchema, Principal => Customer; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PersonalOwner in BankSchema, Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub BusinessOwner in BankSchema, Business => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub BusinessAccount in BankSchema, Business => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub AccountAuthorizedUser in BankSchema,
    Principal => AccountAuthorization; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub AuthorizationAccount in BankSchema,
    AccountAuthorization => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub InstitutionEmployee in BankSchema,
    Institution => EmployeeAssignment; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub AssignmentPrincipal in BankSchema,
    EmployeeAssignment => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub InstitutionAccount in BankSchema, Institution => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub InstitutionCashAccount in BankSchema, Institution => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PaymentSource in BankSchema, PaymentIntent => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PaymentDestination in BankSchema, PaymentIntent => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PaymentBusiness in BankSchema, PaymentIntent => Business; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PaymentInitiator in BankSchema, Principal => PaymentIntent; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PaymentApproval in BankSchema, PaymentIntent => Approval; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ApprovalPrincipal in BankSchema, Approval => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub JournalPosting in BankSchema, JournalEntry => Posting; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub JournalReversal in BankSchema, JournalEntry => JournalEntry; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PostingAccount in BankSchema, Posting => Account; integrity = same_context_unbounded_retain_dangling);
