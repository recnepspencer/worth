use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::identity::data::{KindId, LineageId, StructuralFingerprint};
use crate::transactions::data::RecordRef;
use worth_foundational::facade::AspectKey;

mod custom_name;
mod declared_keys;
pub use custom_name::IdentityBasisName;
pub use declared_keys::DeclaredIdentityKeys;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeRecordIdentity {
    StorageRecord(RecordRef),
    Lineage(LineageId),
    StructuralFingerprint(StructuralFingerprint),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CustomIdentityBasisIdentity {
    pub name: IdentityBasisName,
    pub semantic_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IdentityBasisScope {
    EntityKind(KindId),
    RelationKind(KindId),
    AspectKey(AspectKey),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IdentityBasisKind {
    StorageIdentity,
    LineageIdentity,
    StructuralFingerprint,
    DeclaredKeySet(DeclaredIdentityKeys),
    Custom(CustomIdentityBasisIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IdentityBasisDeclaration {
    pub scope: IdentityBasisScope,
    pub basis: IdentityBasisKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityResolutionReason {
    ExactStorageIdentity,
    ExactLineageIdentity,
    ExactStructuralFingerprint,
    DeclaredBasisUnavailableOnSource,
    DeclaredBasisNoVisibleTargetMatch,
    DeclaredBasisAmbiguousVisibleTargetMatch,
    SchemaDeclaredCorrespondence,
    PreferRicherAspectShape,
    AdvisoryCorrespondenceRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityMatchClass {
    Exact,
    Reconciliable,
    Ambiguous,
    MissingTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityMatchCandidate {
    pub scope: Option<IdentityBasisScope>,
    pub source_record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub source: MergeRecordIdentity,
    pub target: Option<MergeRecordIdentity>,
    pub match_class: IdentityMatchClass,
    pub reason: IdentityResolutionReason,
    pub basis: IdentityBasisKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityDiscoverySummary {
    pub effective_declarations: Arc<[IdentityBasisDeclaration]>,
    pub candidate_count: usize,
    pub exact_match_count: usize,
    pub reconciliable_match_count: usize,
    pub schema_declared_correspondence: SchemaDeclaredCorrespondenceValidationSummary,
    pub ambiguous_match_count: usize,
    pub missing_target_count: usize,
    pub storage_basis_candidate_count: usize,
    pub lineage_basis_candidate_count: usize,
    pub structural_basis_candidate_count: usize,
    pub custom_basis_candidate_count: usize,
    pub candidates: Arc<[IdentityMatchCandidate]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDeclaredCorrespondenceValidationSummary {
    pub candidate_count: usize,
    pub validated_count: usize,
    pub rejected_non_unique_source_count: usize,
    pub rejected_non_unique_target_count: usize,
}
