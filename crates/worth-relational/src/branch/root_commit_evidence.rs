//! A root carries either its original frame or selected checkpoint evidence.
use super::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoint_admission::CheckpointAdmittedCommit;
use crate::history::data::{CanonicalCommitEnvelope, RelationalCommitReceipt};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) enum RelationalRootCommitEvidence {
    Original(Arc<CanonicalCommitEnvelope>),
    Checkpoint(CheckpointAdmittedCommit),
}

impl RelationalRootCommitEvidence {
    pub(crate) fn authoring_branch(&self) -> &crate::history::data::BranchId {
        match self {
            Self::Original(envelope) => &envelope.branch_context,
            Self::Checkpoint(commit) => &commit.index().commit.branch_id,
        }
    }
    pub(crate) fn schema_version(&self) -> crate::schema::data::SchemaVersionId {
        match self {
            Self::Original(envelope) => envelope.schema_version,
            Self::Checkpoint(commit) => commit.index().schema_version,
        }
    }
    pub(crate) fn receipt(&self) -> &RelationalCommitReceipt {
        match self {
            Self::Original(envelope) => &envelope.commit,
            Self::Checkpoint(commit) => &commit.index().commit,
        }
    }
    pub(crate) fn original(&self) -> Option<&Arc<CanonicalCommitEnvelope>> {
        match self {
            Self::Original(envelope) => Some(envelope),
            Self::Checkpoint(_) => None,
        }
    }
    pub(super) fn matches_schema(&self, schema: &RelationalBranchRootSchemaAuthority) -> bool {
        match self {
            Self::Original(envelope) => {
                schema.matches(&envelope.schema_authority)
                    && schema.descriptor_semantics_version()
                        == envelope.descriptor_semantics_version
            }
            Self::Checkpoint(commit) => std::ptr::eq(schema, commit.schema().as_ref()),
        }
    }
    pub(super) fn shares_owner(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Original(left), Self::Original(right)) => Arc::ptr_eq(left, right),
            (Self::Checkpoint(left), Self::Checkpoint(right)) => left.same_owner(right),
            (Self::Original(_), Self::Checkpoint(_)) | (Self::Checkpoint(_), Self::Original(_)) => {
                false
            }
        }
    }
}
