//! Relations of the execution fixture.
use super::*;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, ExternalMapping, Principal> {
        relation(ExecutionRecords, ExternalMapping, Self)
    }
}
pub struct AccountOwner;
impl fragment::Relation for AccountOwner {
    const NAME: &'static str = "AccountOwner";
    type To = Account;
}
impl AccountOwner {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, Account>
    {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct AccountBlocked;
impl fragment::Relation for AccountBlocked {
    const NAME: &'static str = "AccountBlocked";
    type To = Account;
}
impl AccountBlocked {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, Account>
    {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct AccountPrimaryActivity;
impl fragment::Relation for AccountPrimaryActivity {
    const NAME: &'static str = "AccountPrimaryActivity";
    type To = Activity;
}
impl AccountPrimaryActivity {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Account, Activity> {
        relation(ExecutionRecords, Account, Self)
    }
}
pub struct AccountSecondaryActivity;
impl fragment::Relation for AccountSecondaryActivity {
    const NAME: &'static str = "AccountSecondaryActivity";
    type To = Activity;
}
impl AccountSecondaryActivity {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Account, Activity> {
        relation(ExecutionRecords, Account, Self)
    }
}
pub struct AccountAllActivity;
impl fragment::Relation for AccountAllActivity {
    const NAME: &'static str = "AccountAllActivity";
    type To = Activity;
}
impl AccountAllActivity {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Account, Activity> {
        relation(ExecutionRecords, Account, Self)
    }
}
pub struct ActivityAccount;
impl fragment::Relation for ActivityAccount {
    const NAME: &'static str = "ActivityAccount";
    type To = Account;
}
impl ActivityAccount {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Activity, Account> {
        relation(ExecutionRecords, Activity, Self)
    }
}
pub struct CapabilityElevationRequester;
impl fragment::Relation for CapabilityElevationRequester {
    const NAME: &'static str = "CapabilityElevationRequester";
    type To = CapabilityElevation;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityElevationRequester {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityElevation> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityElevationApprover;
impl fragment::Relation for CapabilityElevationApprover {
    const NAME: &'static str = "CapabilityElevationApprover";
    type To = CapabilityElevation;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityElevationApprover {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityElevation> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityElevationGrant;
impl fragment::Relation for CapabilityElevationGrant {
    const NAME: &'static str = "CapabilityElevationGrant";
    type To = CapabilityGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityElevationGrant {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityElevation, CapabilityGrant>
    {
        relation(ElevationRecords, CapabilityElevation, Self)
    }
}
pub struct CapabilityElevationResource;
impl fragment::Relation for CapabilityElevationResource {
    const NAME: &'static str = "CapabilityElevationResource";
    type To = Account;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityElevationResource {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityElevation, Account> {
        relation(ElevationRecords, CapabilityElevation, Self)
    }
}
pub struct CapabilityElevationReview;
impl fragment::Relation for CapabilityElevationReview {
    const NAME: &'static str = "CapabilityElevationReview";
    type To = CapabilityReview;
}
impl CapabilityElevationReview {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityElevation, CapabilityReview>
    {
        relation(ElevationRecords, CapabilityElevation, Self)
    }
}
pub struct CapabilityReviewResource;
impl fragment::Relation for CapabilityReviewResource {
    const NAME: &'static str = "CapabilityReviewResource";
    type To = Account;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityReviewResource {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityReview, Account> {
        relation(ExecutionRecords, CapabilityReview, Self)
    }
}
pub struct CapabilityReviewer;
impl fragment::Relation for CapabilityReviewer {
    const NAME: &'static str = "CapabilityReviewer";
    type To = CapabilityReview;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityReviewer {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityReview> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityGrantee;
impl fragment::Relation for CapabilityGrantee {
    const NAME: &'static str = "CapabilityGrantee";
    type To = CapabilityGrant;
}
impl CapabilityGrantee {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityGrant> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityGrantor;
impl fragment::Relation for CapabilityGrantor {
    const NAME: &'static str = "CapabilityGrantor";
    type To = CapabilityGrant;
}
impl CapabilityGrantor {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityGrant> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityCustodian;
impl fragment::Relation for CapabilityCustodian {
    const NAME: &'static str = "CapabilityCustodian";
    type To = CapabilityGrant;
}
impl CapabilityCustodian {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityGrant> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityResource;
impl fragment::Relation for CapabilityResource {
    const NAME: &'static str = "CapabilityResource";
    type To = Account;
}
impl CapabilityResource {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityGrant, Account> {
        relation(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityRelated;
impl fragment::Relation for CapabilityRelated {
    const NAME: &'static str = "CapabilityRelated";
    type To = Account;
}
impl CapabilityRelated {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityGrant, Account> {
        relation(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityParent;
impl fragment::Relation for CapabilityParent {
    const NAME: &'static str = "CapabilityParent";
    type To = CapabilityGrant;
}
impl CapabilityParent {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityGrant, CapabilityGrant>
    {
        relation(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityExplicitDeny;
impl fragment::Relation for CapabilityExplicitDeny {
    const NAME: &'static str = "CapabilityExplicitDeny";
    type To = Account;
}
impl CapabilityExplicitDeny {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, Account>
    {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityConflictingBeneficiary;
impl fragment::Relation for CapabilityConflictingBeneficiary {
    const NAME: &'static str = "CapabilityConflictingBeneficiary";
    type To = Account;
}
impl CapabilityConflictingBeneficiary {
    pub fn reference() -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, Account>
    {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityRequestActor;
impl fragment::Relation for CapabilityRequestActor {
    const NAME: &'static str = "CapabilityRequestActor";
    type To = CapabilityActionRecord;
}
impl CapabilityRequestActor {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityActionRecord>
    {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityPriorActor;
impl fragment::Relation for CapabilityPriorActor {
    const NAME: &'static str = "CapabilityPriorActor";
    type To = CapabilityActionRecord;
}
impl CapabilityPriorActor {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, Principal, CapabilityActionRecord>
    {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct CapabilityActionResource;
impl fragment::Relation for CapabilityActionResource {
    const NAME: &'static str = "CapabilityActionResource";
    type To = Account;
}
impl CapabilityActionResource {
    pub fn reference(
    ) -> ApplicationRelationRef<IdentityExecutionSchema, Self, CapabilityActionRecord, Account>
    {
        relation(ExecutionRecords, CapabilityActionRecord, Self)
    }
}
