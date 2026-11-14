//! Adoption DATA is folded in the common journal; World reissues live recovery.
use super::*;
use worth_foundational::facade::DurableCompositeCommitIdentity;

pub(super) mod codec;
pub(super) mod fold;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryBranchAdoptionObligation {
    pub attempt: DurableCompositePublicationAttemptIdentity,
    pub source: CanonicalDigestId,
    pub target: CanonicalDigestId,
    pub branch: String,
    pub lifecycle: u64,
    pub generation: u64,
    #[serde(with = "codec::commit")]
    pub expected: DurableCompositeCommitIdentity,
    pub selected_entity_count: u64,
    pub migration: Option<(String, u64)>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QueryAdoptionRecord {
    RetainBranch(QueryBranchAdoptionObligation),
    RetainStopped(QueryStoppedBranchSetObligation),
    ReleaseStopped {
        attempt: DurableCompositePublicationAttemptIdentity,
    },
    ReleaseBranch {
        attempt: DurableCompositePublicationAttemptIdentity,
    },
}
impl From<QueryAdoptionRecord> for QueryObligationRecord {
    fn from(record: QueryAdoptionRecord) -> Self {
        Self::Adoption(record)
    }
}

/// Descriptions of the completed prefix grant no live publication authority.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryAdoptionPerformedDescription {
    pub branch: String,
    pub lifecycle: u64,
    pub source: CanonicalDigestId,
    pub target: CanonicalDigestId,
    #[serde(with = "codec::commit")]
    pub commit: DurableCompositeCommitIdentity,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryStoppedBranchSetObligation {
    pub attempt: DurableCompositePublicationAttemptIdentity,
    pub target: CanonicalDigestId,
    /// Ordered branch name, lifecycle and canonical source meaning.
    pub remaining: Vec<(String, u64, CanonicalDigestId)>,
    pub performed: Vec<QueryAdoptionPerformedDescription>,
    /// Diagnostic only: the final disposition is always an acknowledged no-effect.
    pub stop_diagnostic: String,
    pub selection_work: u64,
}
