use super::*;

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<BodyModel, ExternalMapping, Self> {
        aspect(BodyModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for BodyModelBindings {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .equality_queryable(true)
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
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<BodyModel, Principal, Self> {
        aspect(BodyModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
        .equality_queryable(true)
        .read_only()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct DocumentFacts;
impl Aspect for DocumentFacts {
    const NAME: &'static str = "DocumentFacts";
    type Row = DocumentFactsValue;
}
impl DocumentFacts {
    pub fn reference() -> AspectSelection<BodyModel, Document, Self> {
        aspect(BodyModel, Document, Self)
    }
}
pub struct DocumentFactsValue {
    pub document_identity_field: String,
    pub document_retention_field: u64,
    pub document_audit_source_field: u64,
}
impl fragment::Record<DocumentFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<DocumentFactsValue, Self> {
        field("DocumentIdentityField", |row: &DocumentFactsValue| {
            row.document_identity_field.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("DocumentRetentionField", |row: &DocumentFactsValue| {
            row.document_retention_field
        })
        .equality_queryable(true)
        .and("DocumentAuditSourceField", |row: &DocumentFactsValue| {
            row.document_audit_source_field
        })
        .equality_queryable(true)
        .build(
            |document_identity_field,
             document_retention_field,
             document_audit_source_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DocumentFactsValue {
                    document_identity_field,
                    document_retention_field,
                    document_audit_source_field,
                })
            },
        )
    }
}
pub struct DocumentDraftFacts;
impl Aspect for DocumentDraftFacts {
    const NAME: &'static str = "DocumentDraftFacts";
    type Row = DocumentDraftFactsValue;
}
impl DocumentDraftFacts {
    pub fn reference() -> AspectSelection<BodyModel, Document, Self> {
        aspect(BodyModel, Document, Self)
    }
}
pub struct DocumentDraftFactsValue {
    pub document_draft_note_field: Option<DraftNote>,
}
impl fragment::Record<DocumentDraftFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<DocumentDraftFactsValue, Self> {
        field("DocumentDraftNoteField", |row: &DocumentDraftFactsValue| {
            row.document_draft_note_field.clone()
        })
        .build(
            |document_draft_note_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DocumentDraftFactsValue {
                    document_draft_note_field,
                })
            },
        )
    }
}
pub struct AuditFacts;
impl Aspect for AuditFacts {
    const NAME: &'static str = "AuditFacts";
    type Row = AuditFactsValue;
}
impl AuditFacts {
    pub fn reference() -> AspectSelection<BodyModel, Audit, Self> {
        aspect(BodyModel, Audit, Self)
    }
}
pub struct AuditFactsValue {
    pub audit_identity_field: String,
    pub audit_retention_field: u64,
    pub audit_prior_retention_field: u64,
}
impl fragment::Record<AuditFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<AuditFactsValue, Self> {
        field("AuditIdentityField", |row: &AuditFactsValue| {
            row.audit_identity_field.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("AuditRetentionField", |row: &AuditFactsValue| {
            row.audit_retention_field
        })
        .equality_queryable(true)
        .read_only()
        .and("AuditPriorRetentionField", |row: &AuditFactsValue| {
            row.audit_prior_retention_field
        })
        .equality_queryable(true)
        .read_only()
        .build(
            |audit_identity_field,
             audit_retention_field,
             audit_prior_retention_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AuditFactsValue {
                    audit_identity_field,
                    audit_retention_field,
                    audit_prior_retention_field,
                })
            },
        )
    }
}
pub struct AuditActorFacts;
impl Aspect for AuditActorFacts {
    const NAME: &'static str = "AuditActorFacts";
    type Row = AuditActorFactsValue;
}
impl AuditActorFacts {
    pub fn reference() -> AspectSelection<BodyModel, Audit, Self> {
        aspect(BodyModel, Audit, Self)
    }
}
pub struct AuditActorFactsValue {
    pub audit_actor_field: u64,
}
impl fragment::Record<AuditActorFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<AuditActorFactsValue, Self> {
        field("AuditActorField", |row: &AuditActorFactsValue| {
            row.audit_actor_field
        })
        .equality_queryable(true)
        .read_only()
        .build(|audit_actor_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AuditActorFactsValue { audit_actor_field })
        })
    }
}
pub struct SnapshotFacts;
impl Aspect for SnapshotFacts {
    const NAME: &'static str = "SnapshotFacts";
    type Row = SnapshotFactsValue;
}
impl SnapshotFacts {
    pub fn reference() -> AspectSelection<BodyModel, Snapshot, Self> {
        aspect(BodyModel, Snapshot, Self)
    }
}
pub struct SnapshotFactsValue {
    pub snapshot_identity: u64,
}
impl fragment::Record<SnapshotFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<SnapshotFactsValue, Self> {
        field("SnapshotIdentity", |row: &SnapshotFactsValue| {
            row.snapshot_identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|snapshot_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(SnapshotFactsValue { snapshot_identity })
        })
    }
}
pub struct TenantFacts;
impl Aspect for TenantFacts {
    const NAME: &'static str = "TenantFacts";
    type Row = TenantFactsValue;
}
impl TenantFacts {
    pub fn reference() -> AspectSelection<BodyModel, Tenant, Self> {
        aspect(BodyModel, Tenant, Self)
    }
}
pub struct TenantFactsValue {
    pub tenant_identity: u64,
}
impl fragment::Record<TenantFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<TenantFactsValue, Self> {
        field("TenantIdentity", |row: &TenantFactsValue| {
            row.tenant_identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|tenant_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(TenantFactsValue { tenant_identity })
        })
    }
}
