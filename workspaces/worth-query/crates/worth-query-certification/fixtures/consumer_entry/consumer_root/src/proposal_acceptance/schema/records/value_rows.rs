use super::*;

pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<ProposalRecords, ExternalMapping, Self> {
        aspect(ProposalRecords, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &ExternalIdentityValue| {
            row.mapping_status
        })
        .equality_queryable(true)
        .build(
            |external_identity_field, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status,
                })
            },
        )
    }
}
#[path = "value_rows/principal.rs"]
mod principal;
pub use principal::PrincipalFacts;

pub struct RegistryFacts;
impl fragment::Aspect for RegistryFacts {
    const NAME: &'static str = "RegistryFacts";
    type Row = RegistryFactsValue;
}
impl RegistryFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, Registry, Self> {
        aspect(ProposalRecords, Registry, Self)
    }
}
pub struct RegistryFactsValue {
    pub registry_name: String,
}
impl fragment::Record<RegistryFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<RegistryFactsValue, Self> {
        fragment::field("RegistryName", |row: &RegistryFactsValue| {
            row.registry_name.clone()
        })
        .equality_queryable(true)
        .build(|registry_name, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(RegistryFactsValue { registry_name })
        })
    }
}
pub struct ModelFacts;
impl fragment::Aspect for ModelFacts {
    const NAME: &'static str = "ModelFacts";
    type Row = ModelFactsValue;
}
impl ModelFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, Model, Self> {
        aspect(ProposalRecords, Model, Self)
    }
}
pub struct ModelFactsValue {
    pub model_name: String,
}
impl fragment::Record<ModelFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<ModelFactsValue, Self> {
        fragment::field("ModelName", |row: &ModelFactsValue| row.model_name.clone())
            .equality_queryable(true)
            .build(|model_name, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ModelFactsValue { model_name })
            })
    }
}
pub struct TargetFacts;
impl fragment::Aspect for TargetFacts {
    const NAME: &'static str = "TargetFacts";
    type Row = TargetFactsValue;
}
impl TargetFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, Target, Self> {
        aspect(ProposalRecords, Target, Self)
    }
}
pub struct TargetFactsValue {
    pub target_name: String,
    pub retention: u64,
}
impl fragment::Record<TargetFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<TargetFactsValue, Self> {
        fragment::field("TargetName", |row: &TargetFactsValue| {
            row.target_name.clone()
        })
        .equality_queryable(true)
        .and("Retention", |row: &TargetFactsValue| row.retention)
        .equality_queryable(true)
        .build(|target_name, retention, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(TargetFactsValue {
                target_name,
                retention,
            })
        })
    }
}
pub struct ProposalFacts;
impl fragment::Aspect for ProposalFacts {
    const NAME: &'static str = "ProposalFacts";
    type Row = ProposalFactsValue;
}
impl ProposalFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, Proposal, Self> {
        aspect(ProposalRecords, Proposal, Self)
    }
}
pub struct ProposalFactsValue {
    pub proposal_name: String,
    pub proposal_data: StoredProposal,
}
impl fragment::Record<ProposalFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<ProposalFactsValue, Self> {
        fragment::field("ProposalName", |row: &ProposalFactsValue| {
            row.proposal_name.clone()
        })
        .equality_queryable(true)
        .and("ProposalData", |row: &ProposalFactsValue| {
            row.proposal_data.clone()
        })
        .equality_queryable(true)
        .build(
            |proposal_name, proposal_data, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProposalFactsValue {
                    proposal_name,
                    proposal_data,
                })
            },
        )
    }
}
pub struct MutationGrantFacts;
impl fragment::Aspect for MutationGrantFacts {
    const NAME: &'static str = "MutationGrantFacts";
    type Row = MutationGrantFactsValue;
}
impl MutationGrantFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, MutationGrant, Self> {
        aspect(ProposalRecords, MutationGrant, Self)
    }
}
pub struct MutationGrantFactsValue {
    pub grant_identity: String,
    pub grant_action: u64,
    pub grant_purpose: u64,
    pub grant_status: u64,
    pub grant_scope: String,
    pub grant_not_before: u64,
    pub grant_not_after: u64,
    pub grant_depth: u64,
}
impl fragment::Record<MutationGrantFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<MutationGrantFactsValue, Self> {
        fragment::field("GrantIdentity", |row: &MutationGrantFactsValue| {
            row.grant_identity.clone()
        })
        .equality_queryable(true)
        .and("GrantAction", |row: &MutationGrantFactsValue| {
            row.grant_action
        })
        .equality_queryable(true)
        .read_only()
        .and("GrantPurpose", |row: &MutationGrantFactsValue| {
            row.grant_purpose
        })
        .equality_queryable(true)
        .read_only()
        .and("GrantStatus", |row: &MutationGrantFactsValue| {
            row.grant_status
        })
        .equality_queryable(true)
        .and("GrantScope", |row: &MutationGrantFactsValue| {
            row.grant_scope.clone()
        })
        .equality_queryable(true)
        .and("GrantNotBefore", |row: &MutationGrantFactsValue| {
            row.grant_not_before
        })
        .equality_queryable(true)
        .read_only()
        .and("GrantNotAfter", |row: &MutationGrantFactsValue| {
            row.grant_not_after
        })
        .equality_queryable(true)
        .read_only()
        .and("GrantDepth", |row: &MutationGrantFactsValue| {
            row.grant_depth
        })
        .equality_queryable(true)
        .read_only()
        .build(
            |grant_identity,
             grant_action,
             grant_purpose,
             grant_status,
             grant_scope,
             grant_not_before,
             grant_not_after,
             grant_depth,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MutationGrantFactsValue {
                    grant_identity,
                    grant_action,
                    grant_purpose,
                    grant_status,
                    grant_scope,
                    grant_not_before,
                    grant_not_after,
                    grant_depth,
                })
            },
        )
    }
}
pub struct ClaimFacts;
impl fragment::Aspect for ClaimFacts {
    const NAME: &'static str = "ClaimFacts";
    type Row = ClaimFactsValue;
}
impl ClaimFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, AcceptanceClaim, Self> {
        aspect(ProposalRecords, AcceptanceClaim, Self)
    }
}
pub struct ClaimFactsValue {
    pub claim_name: String,
    pub claim_data: StoredClaim,
}
impl fragment::Record<ClaimFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<ClaimFactsValue, Self> {
        fragment::field("ClaimName", |row: &ClaimFactsValue| row.claim_name.clone())
            .equality_queryable(true)
            .and("ClaimData", |row: &ClaimFactsValue| row.claim_data.clone())
            .equality_queryable(true)
            .build(|claim_name, claim_data, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ClaimFactsValue {
                    claim_name,
                    claim_data,
                })
            })
    }
}
pub struct ResultFacts;
impl fragment::Aspect for ResultFacts {
    const NAME: &'static str = "ResultFacts";
    type Row = ResultFactsValue;
}
impl ResultFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, AcceptanceResult, Self> {
        aspect(ProposalRecords, AcceptanceResult, Self)
    }
}
pub struct ResultFactsValue {
    pub result_name: String,
    pub result_data: StoredResult,
}
impl fragment::Record<ResultFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<ResultFactsValue, Self> {
        fragment::field("ResultName", |row: &ResultFactsValue| {
            row.result_name.clone()
        })
        .equality_queryable(true)
        .and("ResultData", |row: &ResultFactsValue| {
            row.result_data.clone()
        })
        .equality_queryable(true)
        .build(
            |result_name, result_data, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ResultFactsValue {
                    result_name,
                    result_data,
                })
            },
        )
    }
}
pub struct GrantFacts;
impl fragment::Aspect for GrantFacts {
    const NAME: &'static str = "GrantFacts";
    type Row = GrantFactsValue;
}
impl GrantFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, Grant, Self> {
        aspect(ProposalRecords, Grant, Self)
    }
}
pub struct GrantFactsValue {
    pub grant_identity: String,
    pub grant_action: u64,
    pub grant_read_kind: u64,
    pub grant_purpose: u64,
    pub grant_status: u64,
    pub grant_read_status: u64,
    pub grant_scope: String,
    pub grant_not_before: u64,
    pub grant_not_after: u64,
    pub grant_depth: u64,
    pub grant_disclosure: u64,
}
impl fragment::Record<GrantFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<GrantFactsValue, Self> {
        fragment::field("GrantIdentity", |row: &GrantFactsValue| {
            row.grant_identity.clone()
        })
        .equality_queryable(true)
        .and("GrantAction", |row: &GrantFactsValue| row.grant_action)
        .equality_queryable(true)
        .read_only()
        .and("GrantReadKind", |row: &GrantFactsValue| row.grant_read_kind)
        .equality_queryable(true)
        .read_only()
        .and("GrantPurpose", |row: &GrantFactsValue| row.grant_purpose)
        .equality_queryable(true)
        .read_only()
        .and("GrantStatus", |row: &GrantFactsValue| row.grant_status)
        .equality_queryable(true)
        .and("GrantReadStatus", |row: &GrantFactsValue| {
            row.grant_read_status
        })
        .equality_queryable(true)
        .and("GrantScope", |row: &GrantFactsValue| {
            row.grant_scope.clone()
        })
        .equality_queryable(true)
        .and("GrantNotBefore", |row: &GrantFactsValue| {
            row.grant_not_before
        })
        .equality_queryable(true)
        .read_only()
        .and("GrantNotAfter", |row: &GrantFactsValue| row.grant_not_after)
        .equality_queryable(true)
        .read_only()
        .and("GrantDepth", |row: &GrantFactsValue| row.grant_depth)
        .equality_queryable(true)
        .read_only()
        .and("GrantDisclosure", |row: &GrantFactsValue| {
            row.grant_disclosure
        })
        .equality_queryable(true)
        .read_only()
        .build(
            |grant_identity,
             grant_action,
             grant_read_kind,
             grant_purpose,
             grant_status,
             grant_read_status,
             grant_scope,
             grant_not_before,
             grant_not_after,
             grant_depth,
             grant_disclosure,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(GrantFactsValue {
                    grant_identity,
                    grant_action,
                    grant_read_kind,
                    grant_purpose,
                    grant_status,
                    grant_read_status,
                    grant_scope,
                    grant_not_before,
                    grant_not_after,
                    grant_depth,
                    grant_disclosure,
                })
            },
        )
    }
}
