//! Entities of the execution fixture.
use super::*;
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(ExecutionRecords, Self)
    }
}
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = (
        (
            AccountOwner,
            AccountBlocked,
            CapabilityElevationRequester,
            CapabilityElevationApprover,
            CapabilityReviewer,
            CapabilityGrantee,
            CapabilityGrantor,
            CapabilityCustodian,
        ),
        (
            CapabilityExplicitDeny,
            CapabilityConflictingBeneficiary,
            CapabilityRequestActor,
            CapabilityPriorActor,
        ),
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(PrincipalRecords, Self)
    }
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountPolicy, Option<AccountAnnotations>, AccountMembership);
    type Relations = (
        AccountPrimaryActivity,
        AccountSecondaryActivity,
        AccountAllActivity,
    );
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(ExecutionRecords, Self)
    }
}
pub struct Activity;
impl fragment::Entity for Activity {
    const NAME: &'static str = "Activity";
    type Aspects = (ActivityFacts,);
    type Relations = (ActivityAccount,);
}
impl Activity {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(ExecutionRecords, Self)
    }
}
pub struct CapabilityElevation;
impl fragment::Entity for CapabilityElevation {
    const NAME: &'static str = "CapabilityElevation";
    type Aspects = (CapabilityElevationFacts,);
    type Relations = (
        CapabilityElevationGrant,
        CapabilityElevationResource,
        CapabilityElevationReview,
    );
}
impl CapabilityElevation {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(ElevationRecords, Self)
    }
}
pub struct CapabilityReview;
impl fragment::Entity for CapabilityReview {
    const NAME: &'static str = "CapabilityReview";
    type Aspects = (CapabilityReviewFacts,);
    type Relations = (CapabilityReviewResource,);
}
impl CapabilityReview {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(ExecutionRecords, Self)
    }
}
pub struct CapabilityGrant;
impl fragment::Entity for CapabilityGrant {
    const NAME: &'static str = "CapabilityGrant";
    type Aspects = (CapabilityFacts,);
    type Relations = (CapabilityResource, CapabilityRelated, CapabilityParent);
}
impl CapabilityGrant {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(GrantRecords, Self)
    }
}
pub struct CapabilityActionRecord;
impl fragment::Entity for CapabilityActionRecord {
    const NAME: &'static str = "CapabilityActionRecord";
    type Aspects = (CapabilityActionRecordFacts,);
    type Relations = (CapabilityActionResource,);
}
impl CapabilityActionRecord {
    pub fn reference() -> ApplicationEntityRef<IdentityExecutionSchema, Self> {
        entity(ExecutionRecords, Self)
    }
}
