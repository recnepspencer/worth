use worth_query_decl::facade::application_query::{
    ApplicationQueryResultRelationRef, ExactlyOneResult, ForwardResultTraversal, ManyResults,
    OptionalOneResult, ReverseResultTraversal,
};

use crate::schema::{
    Account, AssignmentPrincipal, BankSchema, Branch, CapabilityAccount, CapabilityBranch,
    CapabilityEstate, CapabilityGrant, CapabilityGrantee, CapabilityGrantor, CapabilityInstitution,
    CapabilityParent, EmergencyAccess, EmergencyApprover, EmergencyGrant, EmergencyRequester,
    EmergencyReview, EmployeeAssignment, EstateAssignment, EstateBeneficiary, EstateCase,
    Institution, MandatoryReview, Principal, ReviewEstate, ReviewPrincipal,
};

use super::governance::EstateGovernanceQuery;

pub(super) struct EstateBeneficiariesRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateBeneficiariesRelationSlot => "EstateGovernanceEstateBeneficiariesRelationSlot");
pub(super) struct EstateAssignmentsRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateAssignmentsRelationSlot => "EstateGovernanceEstateAssignmentsRelationSlot");
pub(super) struct AssignmentPrincipalRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentPrincipalRelationSlot => "EstateGovernanceAssignmentPrincipalRelationSlot");
pub(super) struct EstateCapabilitiesRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateCapabilitiesRelationSlot => "EstateGovernanceEstateCapabilitiesRelationSlot");
pub(super) struct CapabilityGranteeRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityGranteeRelationSlot => "EstateGovernanceCapabilityGranteeRelationSlot");
pub(super) struct CapabilityGrantorRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityGrantorRelationSlot => "EstateGovernanceCapabilityGrantorRelationSlot");
pub(super) struct CapabilityAccountRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityAccountRelationSlot => "EstateGovernanceCapabilityAccountRelationSlot");
pub(super) struct CapabilityInstitutionRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityInstitutionRelationSlot => "EstateGovernanceCapabilityInstitutionRelationSlot");
pub(super) struct CapabilityBranchRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityBranchRelationSlot => "EstateGovernanceCapabilityBranchRelationSlot");
pub(super) struct CapabilityParentRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityParentRelationSlot => "EstateGovernanceCapabilityParentRelationSlot");
pub(super) struct CapabilityEmergenciesRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CapabilityEmergenciesRelationSlot => "EstateGovernanceCapabilityEmergenciesRelationSlot");
pub(super) struct EmergencyRequesterRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(EmergencyRequesterRelationSlot => "EstateGovernanceEmergencyRequesterRelationSlot");
pub(super) struct EmergencyApproverRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(EmergencyApproverRelationSlot => "EstateGovernanceEmergencyApproverRelationSlot");
pub(super) struct EmergencyReviewRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(EmergencyReviewRelationSlot => "EstateGovernanceEmergencyReviewRelationSlot");
pub(super) struct ReviewEstateRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewEstateRelationSlot => "EstateGovernanceReviewEstateRelationSlot");
pub(super) struct ReviewReviewerRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewReviewerRelationSlot => "EstateGovernanceReviewReviewerRelationSlot");

macro_rules! reverse_many {
    ($name:ident, $slot:ty, $relation:ty, $from:ty, $to:ty, $alias:literal) => {
        pub(super) fn $name() -> ApplicationQueryResultRelationRef<
            EstateGovernanceQuery,
            $slot,
            BankSchema,
            $relation,
            $from,
            $to,
            ReverseResultTraversal,
            ManyResults,
        > {
            ApplicationQueryResultRelationRef::reverse_many($alias, <$relation>::reference())
        }
    };
}

macro_rules! reverse_one {
    ($name:ident, $slot:ty, $relation:ty, $from:ty, $to:ty, $cardinality:ty, $method:ident, $alias:literal) => {
        pub(super) fn $name() -> ApplicationQueryResultRelationRef<
            EstateGovernanceQuery,
            $slot,
            BankSchema,
            $relation,
            $from,
            $to,
            ReverseResultTraversal,
            $cardinality,
        > {
            ApplicationQueryResultRelationRef::$method($alias, <$relation>::reference())
        }
    };
}

reverse_many!(
    estate_beneficiaries,
    EstateBeneficiariesRelationSlot,
    EstateBeneficiary,
    Principal,
    EstateCase,
    "beneficiaries"
);
reverse_many!(
    estate_assignments,
    EstateAssignmentsRelationSlot,
    EstateAssignment,
    EmployeeAssignment,
    EstateCase,
    "assignments"
);
reverse_many!(
    estate_capabilities,
    EstateCapabilitiesRelationSlot,
    CapabilityEstate,
    CapabilityGrant,
    EstateCase,
    "capabilities"
);
reverse_many!(
    capability_emergencies,
    CapabilityEmergenciesRelationSlot,
    EmergencyGrant,
    EmergencyAccess,
    CapabilityGrant,
    "emergencies"
);
reverse_one!(
    capability_grantee,
    CapabilityGranteeRelationSlot,
    CapabilityGrantee,
    Principal,
    CapabilityGrant,
    ExactlyOneResult,
    reverse_one,
    "grantee"
);

pub(super) fn capability_account() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    CapabilityAccountRelationSlot,
    BankSchema,
    CapabilityAccount,
    CapabilityGrant,
    Account,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("account", CapabilityAccount::reference())
}

pub(super) fn capability_institution() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    CapabilityInstitutionRelationSlot,
    BankSchema,
    CapabilityInstitution,
    CapabilityGrant,
    Institution,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one(
        "institution",
        CapabilityInstitution::reference(),
    )
}

pub(super) fn capability_branch() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    CapabilityBranchRelationSlot,
    BankSchema,
    CapabilityBranch,
    CapabilityGrant,
    Branch,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("branch", CapabilityBranch::reference())
}

pub(super) fn capability_parent() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    CapabilityParentRelationSlot,
    BankSchema,
    CapabilityParent,
    CapabilityGrant,
    CapabilityGrant,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("parent", CapabilityParent::reference())
}

pub(super) fn emergency_review() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    EmergencyReviewRelationSlot,
    BankSchema,
    EmergencyReview,
    EmergencyAccess,
    MandatoryReview,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("review", EmergencyReview::reference())
}

pub(super) fn review_estate() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    ReviewEstateRelationSlot,
    BankSchema,
    ReviewEstate,
    MandatoryReview,
    EstateCase,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("estate", ReviewEstate::reference())
}

pub(super) fn review_reviewer() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    ReviewReviewerRelationSlot,
    BankSchema,
    ReviewPrincipal,
    Principal,
    MandatoryReview,
    ReverseResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_optional("reviewer", ReviewPrincipal::reference())
}
reverse_one!(
    capability_grantor,
    CapabilityGrantorRelationSlot,
    CapabilityGrantor,
    Principal,
    CapabilityGrant,
    ExactlyOneResult,
    reverse_one,
    "grantor"
);
reverse_one!(
    emergency_requester,
    EmergencyRequesterRelationSlot,
    EmergencyRequester,
    Principal,
    EmergencyAccess,
    ExactlyOneResult,
    reverse_one,
    "requester"
);
reverse_one!(
    emergency_approver,
    EmergencyApproverRelationSlot,
    EmergencyApprover,
    Principal,
    EmergencyAccess,
    OptionalOneResult,
    reverse_optional,
    "approver"
);

pub(super) fn assignment_principal() -> ApplicationQueryResultRelationRef<
    EstateGovernanceQuery,
    AssignmentPrincipalRelationSlot,
    BankSchema,
    AssignmentPrincipal,
    EmployeeAssignment,
    Principal,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("principal", AssignmentPrincipal::reference())
}
