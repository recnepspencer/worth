use super::*;

pub type ExternalIdentityField = FieldSelection<
    PublicationRecords,
    ExternalMapping,
    ExternalIdentity,
    worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    PublicationRecords,
    ExternalMapping,
    ExternalIdentity,
    worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField =
    FieldSelection<PublicationRecords, Principal, PrincipalFacts, u64>;
pub type ResourceIdentityField =
    FieldSelection<PublicationRecords, Resource, ResourceFacts, String>;
pub type ResourceWorkflowField =
    FieldSelection<PublicationRecords, Resource, ResourceFacts, String>;
pub type ResourceLabelField = FieldSelection<PublicationRecords, Resource, ResourceFacts, String>;
pub type GrantIdentityField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, GrantKey>;
pub type GrantActionField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, String>;
pub type GrantPurposeField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, String>;
pub type GrantStatusField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, String>;
pub type GrantWorkflowField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, String>;
pub type GrantNotBeforeField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, u64>;
pub type GrantNotAfterField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, u64>;
pub type GrantDelegationLimitField = FieldSelection<GrantRecords, CapabilityGrant, GrantFacts, u64>;
pub type ActionRecordIdentityField =
    FieldSelection<PublicationRecords, ActionRecord, ActionRecordFacts, String>;
