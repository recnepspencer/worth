use super::*;

pub type SnapshotIdentity = FieldSelection<InputModel, Snapshot, SnapshotFacts, String>;
pub type SnapshotValue = FieldSelection<InputModel, Snapshot, SnapshotFacts, u64>;
pub type SnapshotNote = FieldSelection<InputModel, Snapshot, SnapshotFacts, Note>;
pub type SnapshotLabel = FieldSelection<InputModel, Snapshot, SnapshotFacts, String>;
pub type SnapshotKind = FieldSelection<InputModel, Snapshot, SnapshotFacts, String>;
pub type ExternalIdentity =
    FieldSelection<InputModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<InputModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<InputModel, Principal, PrincipalFacts, u64>;
pub type ParentIdentity = FieldSelection<InputModel, Parent, ParentFacts, String>;
pub type Company = FieldSelection<InputModel, Parent, ParentFacts, String>;
pub type ParentRevision = FieldSelection<InputModel, Parent, ParentFacts, u64>;
pub type RowIdentity = FieldSelection<InputModel, Row, RowFacts, String>;
pub type RowValue = FieldSelection<InputModel, Row, RowFacts, u64>;
pub type AuditIdentity = FieldSelection<InputModel, Audit, AuditFacts, String>;
pub type AuditValue = FieldSelection<InputModel, Audit, AuditFacts, u64>;
pub type SignedIdentity = FieldSelection<InputModel, Evidence, Facts, String>;
pub type Actor = FieldSelection<InputModel, Evidence, Facts, u64>;
pub type Provenance =
    FieldSelection<InputModel, Evidence, Facts, ApplicationSigningGraphSubjectProvenance>;
pub type Record =
    FieldSelection<InputModel, Consequence, ConsequenceFacts, ApplicationSigningSupersession>;
pub type JournalName = FieldSelection<InputModel, Journal, JournalFacts, String>;
pub type JournalRevision = FieldSelection<InputModel, Journal, JournalFacts, u64>;
pub type JournalActor = FieldSelection<InputModel, Journal, JournalFacts, u64>;
pub type SignedValue = FieldSelection<InputModel, GraphEvidence, GraphFacts, u64>;
pub type SignedRevision = FieldSelection<InputModel, GraphEvidence, GraphFacts, u64>;
pub type SignedCompany = FieldSelection<InputModel, GraphEvidence, GraphFacts, String>;
pub type GraphActor = FieldSelection<InputModel, GraphEvidence, GraphFacts, u64>;
pub type GraphProvenance =
    FieldSelection<InputModel, GraphEvidence, GraphFacts, ApplicationSigningGraphSubjectProvenance>;
pub type ScopeValue = FieldSelection<InputModel, ScopeEvidence, ScopeFacts, u64>;
pub type ScopeActor = FieldSelection<InputModel, ScopeEvidence, ScopeFacts, u64>;
pub type ScopeProvenance =
    FieldSelection<InputModel, ScopeEvidence, ScopeFacts, ApplicationSigningSubjectProvenance>;
pub type CellIdentity = FieldSelection<InputModel, Cell, CellFacts, u64>;
pub type CellKind = FieldSelection<InputModel, Cell, CellFacts, u64>;
pub type CellNote = FieldSelection<InputModel, Cell, CellFacts, Note>;
pub type DetailLabel = FieldSelection<InputModel, Detail, DetailFacts, String>;
