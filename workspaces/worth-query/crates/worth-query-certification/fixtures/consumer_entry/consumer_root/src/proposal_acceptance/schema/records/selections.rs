use super::*;

pub type ExternalIdentityField = FieldSelection<
    ProposalRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatus = FieldSelection<
    ProposalRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentity = FieldSelection<ProposalRecords, Principal, PrincipalFacts, u64>;
pub type RegistryName = FieldSelection<ProposalRecords, Registry, RegistryFacts, String>;
pub type ModelName = FieldSelection<ProposalRecords, Model, ModelFacts, String>;
pub type TargetName = FieldSelection<ProposalRecords, Target, TargetFacts, String>;
pub type Retention = FieldSelection<ProposalRecords, Target, TargetFacts, u64>;
pub type ProposalName = FieldSelection<ProposalRecords, Proposal, ProposalFacts, String>;
pub type ProposalData = FieldSelection<ProposalRecords, Proposal, ProposalFacts, StoredProposal>;
pub type MutationGrantIdentity =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, String>;
pub type MutationGrantAction =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, u64>;
pub type MutationGrantPurpose =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, u64>;
pub type MutationGrantStatus =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, u64>;
pub type MutationGrantScope =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, String>;
pub type MutationGrantNotBefore =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, u64>;
pub type MutationGrantNotAfter =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, u64>;
pub type MutationGrantDepth =
    FieldSelection<ProposalRecords, MutationGrant, MutationGrantFacts, u64>;
pub type ClaimName = FieldSelection<ProposalRecords, AcceptanceClaim, ClaimFacts, String>;
pub type ClaimData = FieldSelection<ProposalRecords, AcceptanceClaim, ClaimFacts, StoredClaim>;
pub type ResultName = FieldSelection<ProposalRecords, AcceptanceResult, ResultFacts, String>;
pub type ResultData = FieldSelection<ProposalRecords, AcceptanceResult, ResultFacts, StoredResult>;
pub type GrantIdentity = FieldSelection<ProposalRecords, Grant, GrantFacts, String>;
pub type GrantAction = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantReadKind = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantPurpose = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantStatus = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantReadStatus = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantScope = FieldSelection<ProposalRecords, Grant, GrantFacts, String>;
pub type GrantNotBefore = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantNotAfter = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantDepth = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
pub type GrantDisclosure = FieldSelection<ProposalRecords, Grant, GrantFacts, u64>;
