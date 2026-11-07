//! One record model owns field lowering and reusable scalar bindings.
#[path = "draft_note.rs"]
mod draft_note;
use super::*;
pub use draft_note::{DraftNote, DraftNoteBinding};
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct BodyModel;
impl Fragment for BodyModel {
    const NAME: &'static str = "body_model";
    type Bindings = BodyModelBindings;
    type Members = (
        ExternalMapping,
        Principal,
        Document,
        Audit,
        Snapshot,
        Tenant,
    );
}
pub struct BodyModelBindings;
impl BindingSet for BodyModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for BodyModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for BodyModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for BodyModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl Scalar<String> for BodyModelBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}

#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{Audit, Document, ExternalMapping, Principal, Snapshot, Tenant};
#[path = "records/value_rows.rs"]
mod value_rows;
pub use value_rows::{
    AuditActorFacts, AuditActorFactsValue, AuditFacts, AuditFactsValue, DocumentDraftFacts,
    DocumentDraftFactsValue, DocumentFacts, DocumentFactsValue, ExternalIdentity,
    ExternalIdentityValue, PrincipalFacts, PrincipalFactsValue, SnapshotFacts, SnapshotFactsValue,
    TenantFacts, TenantFactsValue,
};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    AuditActorField, AuditIdentityField, AuditPriorRetentionField, AuditRetentionField,
    DocumentAuditSourceField, DocumentDraftNoteField, DocumentIdentityField,
    DocumentRetentionField, ExternalIdentityField, MappingStatusField, PrincipalIdentityField,
    SnapshotIdentity, TenantIdentity,
};
#[path = "records/relations.rs"]
mod relations;
pub use relations::{
    AlternateSelection, CurrentSelection, DocumentMembers, DocumentTenant, MappingTarget,
    OwnedRows, TenantDocuments,
};
