use worth_query_decl::facade::worth_query_relation;

use crate::schema::BankSchema;
use crate::schema::{Account, EmployeeAssignment, Institution, Principal};

use super::entities::{
    Branch, CapabilityGrant, DeathNotice, EmergencyAccess, EstateCase, LegalAuthority,
    MandatoryReview,
};

worth_query_relation!(pub BranchInstitution in BankSchema, Branch => Institution; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub DeathNoticeSubject in BankSchema, DeathNotice => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateDeathNotice in BankSchema, EstateCase => DeathNotice; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateDeceased in BankSchema, EstateCase => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateAccount in BankSchema, EstateCase => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateBranch in BankSchema, EstateCase => Branch; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateExecutor in BankSchema, Principal => EstateCase; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateBeneficiary in BankSchema, Principal => EstateCase; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateJointOwner in BankSchema, Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub EstateAuthorizedSigner in BankSchema, Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub EstateAssignment in BankSchema,
    EmployeeAssignment => EstateCase; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub LegalAuthorityEstate in BankSchema,
    LegalAuthority => EstateCase; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub LegalAuthorityHolder in BankSchema,
    LegalAuthority => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityGrantee in BankSchema,
    Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityGrantor in BankSchema,
    Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityEstate in BankSchema,
    CapabilityGrant => EstateCase; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityAccount in BankSchema,
    CapabilityGrant => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityInstitution in BankSchema,
    CapabilityGrant => Institution; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityBranch in BankSchema,
    CapabilityGrant => Branch; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityParent in BankSchema,
    CapabilityGrant => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub EmergencyRequester in BankSchema,
    Principal => EmergencyAccess; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub EmergencyApprover in BankSchema,
    Principal => EmergencyAccess; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub EmergencyGrant in BankSchema,
    EmergencyAccess => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub EmergencyEstate in BankSchema,
    EmergencyAccess => EstateCase; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub EmergencyReview in BankSchema,
    EmergencyAccess => MandatoryReview; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub ReviewPrincipal in BankSchema,
    Principal => MandatoryReview; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub ReviewEstate in BankSchema,
    MandatoryReview => EstateCase; integrity = same_context_unbounded_retain_dangling);
