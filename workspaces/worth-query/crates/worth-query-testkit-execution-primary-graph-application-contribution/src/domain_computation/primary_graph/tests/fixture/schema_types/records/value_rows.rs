//! Value rows of the execution fixture.
use super::*;
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<ExecutionRecords, ExternalMapping, Self> {
        aspect(ExecutionRecords, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalIdentity;
impl fragment::Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityValue;
}
impl PrincipalIdentity {
    pub fn reference() -> AspectSelection<PrincipalRecords, Principal, Self> {
        aspect(PrincipalRecords, Principal, Self)
    }
}
pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalIdentityValue> for PrincipalBindings {
    fn record() -> impl fragment::Shape<PrincipalIdentityValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
        .read_only()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct AccountPolicy;
impl fragment::Aspect for AccountPolicy {
    const NAME: &'static str = "AccountPolicy";
    type Row = AccountPolicyValue;
}
impl AccountPolicy {
    pub fn reference() -> AspectSelection<ExecutionRecords, Account, Self> {
        aspect(ExecutionRecords, Account, Self)
    }
}
pub struct AccountPolicyValue {
    pub account_identity: String,
    pub account_status: String,
    pub account_label: String,
    pub account_note: Option<String>,
    pub account_score: Option<u64>,
}
impl fragment::Record<AccountPolicyValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<AccountPolicyValue, Self> {
        fragment::field("AccountIdentity", |row: &AccountPolicyValue| {
            row.account_identity.clone()
        })
        .read_only()
        .and("AccountStatus", |row: &AccountPolicyValue| {
            row.account_status.clone()
        })
        .and("AccountLabel", |row: &AccountPolicyValue| {
            row.account_label.clone()
        })
        .and("AccountNote", |row: &AccountPolicyValue| {
            row.account_note.clone()
        })
        .and("AccountScore", |row: &AccountPolicyValue| row.account_score)
        .build(
            |account_identity,
             account_status,
             account_label,
             account_note,
             account_score,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountPolicyValue {
                    account_identity,
                    account_status,
                    account_label,
                    account_note,
                    account_score,
                })
            },
        )
    }
}
pub struct AccountAnnotations;
impl fragment::Aspect for AccountAnnotations {
    const NAME: &'static str = "AccountAnnotations";
    type Row = AccountAnnotationsValue;
}
impl AccountAnnotations {
    pub fn reference() -> AspectSelection<ExecutionRecords, Account, Self> {
        aspect(ExecutionRecords, Account, Self)
    }
}
pub struct AccountAnnotationsValue {
    pub account_annotation: Option<AnnotationText>,
}
impl fragment::Record<AccountAnnotationsValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<AccountAnnotationsValue, Self> {
        fragment::field("AccountAnnotation", |row: &AccountAnnotationsValue| {
            row.account_annotation.clone()
        })
        .build(|account_annotation, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AccountAnnotationsValue { account_annotation })
        })
    }
}
pub struct AccountMembership;
impl fragment::Aspect for AccountMembership {
    const NAME: &'static str = "AccountMembership";
    type Row = AccountMembershipValue;
}
impl AccountMembership {
    pub fn reference() -> AspectSelection<ExecutionRecords, Account, Self> {
        aspect(ExecutionRecords, Account, Self)
    }
}
pub struct AccountMembershipValue {
    pub account_membership_tag: String,
}
impl fragment::Record<AccountMembershipValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<AccountMembershipValue, Self> {
        fragment::field("AccountMembershipTag", |row: &AccountMembershipValue| {
            row.account_membership_tag.clone()
        })
        .read_only()
        .build(|account_membership_tag, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AccountMembershipValue {
                account_membership_tag,
            })
        })
    }
}
pub struct ActivityFacts;
impl fragment::Aspect for ActivityFacts {
    const NAME: &'static str = "ActivityFacts";
    type Row = ActivityFactsValue;
}
impl ActivityFacts {
    pub fn reference() -> AspectSelection<ExecutionRecords, Activity, Self> {
        aspect(ExecutionRecords, Activity, Self)
    }
}
pub struct ActivityFactsValue {
    pub activity_identity: String,
    pub activity_sequence: u64,
}
impl fragment::Record<ActivityFactsValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<ActivityFactsValue, Self> {
        fragment::field("ActivityIdentity", |row: &ActivityFactsValue| {
            row.activity_identity.clone()
        })
        .read_only()
        .and("ActivitySequence", |row: &ActivityFactsValue| {
            row.activity_sequence
        })
        .read_only()
        .build(
            |activity_identity, activity_sequence, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ActivityFactsValue {
                    activity_identity,
                    activity_sequence,
                })
            },
        )
    }
}
