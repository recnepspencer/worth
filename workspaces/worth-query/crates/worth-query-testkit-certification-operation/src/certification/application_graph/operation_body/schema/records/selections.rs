use super::*;

pub type AuditActorField = FieldSelection<BodyModel, Audit, AuditActorFacts, u64>;
pub type ExternalIdentityField = FieldSelection<
    BodyModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField =
    FieldSelection<BodyModel, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentityField = FieldSelection<BodyModel, Principal, PrincipalFacts, u64>;
pub type DocumentIdentityField = FieldSelection<BodyModel, Document, DocumentFacts, String>;
pub type DocumentRetentionField = FieldSelection<BodyModel, Document, DocumentFacts, u64>;
pub type DocumentAuditSourceField = FieldSelection<BodyModel, Document, DocumentFacts, u64>;
pub type DocumentDraftNoteField =
    FieldSelection<BodyModel, Document, DocumentDraftFacts, DraftNote>;
pub type AuditIdentityField = FieldSelection<BodyModel, Audit, AuditFacts, String>;
pub type AuditRetentionField = FieldSelection<BodyModel, Audit, AuditFacts, u64>;
pub type AuditPriorRetentionField = FieldSelection<BodyModel, Audit, AuditFacts, u64>;
pub type SnapshotIdentity = FieldSelection<BodyModel, Snapshot, SnapshotFacts, u64>;
pub type TenantIdentity = FieldSelection<BodyModel, Tenant, TenantFacts, u64>;
