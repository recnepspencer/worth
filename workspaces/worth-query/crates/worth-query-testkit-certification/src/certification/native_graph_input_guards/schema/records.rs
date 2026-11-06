//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct InputModel;
impl Fragment for InputModel {
    const NAME: &'static str = "input_model";
    type Bindings = InputModelBindings;
    type Members = (
        (
            Snapshot,
            Mapping,
            Principal,
            Parent,
            Row,
            Audit,
            Evidence,
            Consequence,
        ),
        (Journal, GraphEvidence, ScopeEvidence, Cell, Detail),
    );
}
pub struct InputModelBindings;
impl BindingSet for InputModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for InputModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for InputModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<Note> for InputModelBindings {
    type Binding = NoteCodec;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for InputModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for InputModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<ApplicationSigningGraphSubjectProvenance> for InputModelBindings {
    type Binding = ApplicationSigningGraphSubjectProvenanceBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<ApplicationSigningSupersession> for InputModelBindings {
    type Binding = ApplicationSigningSupersessionBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<ApplicationSigningSubjectProvenance> for InputModelBindings {
    type Binding = ApplicationSigningSubjectProvenanceBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}

#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{
    Audit, Cell, Consequence, Detail, Evidence, GraphEvidence, Journal, Mapping, Parent, Principal,
    Row, ScopeEvidence, Snapshot,
};
#[path = "records/input_rows.rs"]
mod input_rows;
pub use input_rows::{MappingFacts, ParentFacts, PrincipalFacts, RowFacts, SnapshotFacts};
#[path = "records/evidence_rows.rs"]
mod evidence_rows;
pub use evidence_rows::{
    AuditFacts, CellFacts, ConsequenceFacts, DetailFacts, Facts, GraphFacts, JournalFacts,
    ScopeFacts,
};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    Actor, AuditIdentity, AuditValue, CellIdentity, CellKind, CellNote, Company, DetailLabel,
    ExternalIdentity, GraphActor, GraphProvenance, JournalActor, JournalName, JournalRevision,
    MappingStatus, ParentIdentity, ParentRevision, PrincipalIdentity, Provenance, Record,
    RowIdentity, RowValue, ScopeActor, ScopeProvenance, ScopeValue, SignedCompany, SignedIdentity,
    SignedRevision, SignedValue, SnapshotIdentity, SnapshotKind, SnapshotLabel, SnapshotNote,
    SnapshotValue,
};
#[path = "records/relations.rs"]
mod relations;
pub use relations::{
    CellDetails, Consequences, Current, CurrentSnapshot, GraphHistory, History, JournalDetails,
    MappingTarget, ParentJournal, ParentRows, RowCellArchive, RowCells, ScopeHistory,
    SelectedParent,
};
