//! Selections of the execution fixture.
use super::*;
pub type ExternalIdentityField = FieldSelection<
    ExecutionRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type PrincipalIdentityField =
    FieldSelection<PrincipalRecords, Principal, PrincipalIdentity, u64>;
pub type MappingStatusField = FieldSelection<
    ExecutionRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type AccountIdentity = FieldSelection<ExecutionRecords, Account, AccountPolicy, String>;
pub type AccountStatus = FieldSelection<ExecutionRecords, Account, AccountPolicy, String>;
pub type AccountLabel = FieldSelection<ExecutionRecords, Account, AccountPolicy, String>;
pub type AccountNote = FieldSelection<ExecutionRecords, Account, AccountPolicy, String>;
pub type AccountScore = FieldSelection<ExecutionRecords, Account, AccountPolicy, u64>;
pub type AccountMembershipTag =
    FieldSelection<ExecutionRecords, Account, AccountMembership, String>;
pub type AccountAnnotation =
    FieldSelection<ExecutionRecords, Account, AccountAnnotations, AnnotationText>;
pub type ActivityIdentity = FieldSelection<ExecutionRecords, Activity, ActivityFacts, String>;
pub type ActivitySequence = FieldSelection<ExecutionRecords, Activity, ActivityFacts, u64>;
pub type CapabilityElevationIdentity =
    FieldSelection<ElevationRecords, CapabilityElevation, CapabilityElevationFacts, ElevationKey>;
pub type CapabilityElevationReason =
    FieldSelection<ElevationRecords, CapabilityElevation, CapabilityElevationFacts, String>;
pub type CapabilityElevationStatusField = FieldSelection<
    ElevationRecords,
    CapabilityElevation,
    CapabilityElevationFacts,
    CapabilityElevationStatus,
>;
pub type CapabilityElevationNotBefore =
    FieldSelection<ElevationRecords, CapabilityElevation, CapabilityElevationFacts, u64>;
pub type CapabilityElevationNotAfter =
    FieldSelection<ElevationRecords, CapabilityElevation, CapabilityElevationFacts, u64>;
pub type CapabilityElevationClosedAt =
    FieldSelection<ElevationRecords, CapabilityElevation, CapabilityElevationFacts, u64>;
pub type CapabilityReviewIdentity =
    FieldSelection<ExecutionRecords, CapabilityReview, CapabilityReviewFacts, String>;
pub type CapabilityReviewKindField =
    FieldSelection<ExecutionRecords, CapabilityReview, CapabilityReviewFacts, CapabilityReviewKind>;
pub type CapabilityReviewStatusField = FieldSelection<
    ExecutionRecords,
    CapabilityReview,
    CapabilityReviewFacts,
    CapabilityReviewStatus,
>;
pub type CapabilityReviewReviewedAt =
    FieldSelection<ExecutionRecords, CapabilityReview, CapabilityReviewFacts, u64>;
pub type CapabilityIdentity =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, GrantKey>;
pub type CapabilityActionField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, CapabilityAction>;
pub type CapabilityPurposeField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, CapabilityPurpose>;
pub type CapabilityDisclosureField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, CapabilityDisclosure>;
pub type CapabilityAmountField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, u64>;
pub type CapabilityStatusField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, CapabilityStatus>;
pub type CapabilityWorkflowField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, String>;
pub type CapabilityNotBeforeField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, u64>;
pub type CapabilityNotAfterField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, u64>;
pub type CapabilityDelegationLimitField =
    FieldSelection<GrantRecords, CapabilityGrant, CapabilityFacts, u64>;
pub type CapabilityActionRecordIdentity =
    FieldSelection<ExecutionRecords, CapabilityActionRecord, CapabilityActionRecordFacts, String>;
