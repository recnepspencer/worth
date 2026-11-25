//! A root carries either its original frame or selected checkpoint evidence.
use super::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
pub use crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence;
use crate::history::data::{CanonicalCommitEnvelope, RelationalCommitReceipt};
use crate::query::data::QueryPlanEvidenceBasis;
use std::any::Any;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

trait OriginalRootCommitEvidenceBacking: Send + Sync {
    fn as_evidence(&self) -> &RelationalOriginalRootCommitEvidence;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl OriginalRootCommitEvidenceBacking for RelationalOriginalRootCommitEvidence {
    fn as_evidence(&self) -> &RelationalOriginalRootCommitEvidence {
        self
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[derive(Clone)]
pub struct SharedOriginalRootCommitEvidence(Arc<dyn OriginalRootCommitEvidenceBacking>);

impl SharedOriginalRootCommitEvidence {
    pub fn from_arc(evidence: Arc<RelationalOriginalRootCommitEvidence>) -> Self {
        Self(evidence)
    }

    pub fn to_arc(&self) -> Arc<RelationalOriginalRootCommitEvidence> {
        Arc::clone(&self.0)
            .into_any()
            .downcast::<RelationalOriginalRootCommitEvidence>()
            .expect(
                "SharedOriginalRootCommitEvidence always wraps RelationalOriginalRootCommitEvidence",
            )
    }
}

impl Deref for SharedOriginalRootCommitEvidence {
    type Target = RelationalOriginalRootCommitEvidence;

    fn deref(&self) -> &Self::Target {
        OriginalRootCommitEvidenceBacking::as_evidence(&*self.0)
    }
}

impl fmt::Debug for SharedOriginalRootCommitEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum RelationalRootCommitEvidence {
    Original(SharedOriginalRootCommitEvidence),
    Checkpoint(CheckpointAdmittedCommit),
}

impl RelationalRootCommitEvidence {
    pub fn original_from_arc(evidence: Arc<RelationalOriginalRootCommitEvidence>) -> Self {
        Self::Original(SharedOriginalRootCommitEvidence::from_arc(evidence))
    }

    pub fn original_for_request(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        envelope: Arc<CanonicalCommitEnvelope>,
    ) -> Result<Self, super::RelationalBranchRootCaptureDenial> {
        RelationalOriginalRootCommitEvidence::prepare(execution, envelope)
            .map(Self::original_from_arc)
    }

    pub fn original_payload(&self) -> Option<Arc<RelationalOriginalRootCommitEvidence>> {
        match self {
            Self::Original(original) => Some(original.to_arc()),
            Self::Checkpoint(_) => None,
        }
    }

    pub fn authoring_branch(&self) -> &crate::history::data::BranchId {
        match self {
            Self::Original(original) => &original.envelope().branch_context,
            Self::Checkpoint(commit) => &commit.index().commit.branch_id,
        }
    }
    pub fn schema_version(&self) -> crate::schema::data::SchemaVersionId {
        match self {
            Self::Original(original) => original.envelope().schema_version,
            Self::Checkpoint(commit) => commit.index().schema_version,
        }
    }
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        match self {
            Self::Original(original) => &original.envelope().commit,
            Self::Checkpoint(commit) => &commit.index().commit,
        }
    }
    pub fn original(&self) -> Option<&Arc<CanonicalCommitEnvelope>> {
        match self {
            Self::Original(original) => Some(original.envelope()),
            Self::Checkpoint(_) => None,
        }
    }
    pub(super) fn matches_schema(&self, schema: &RelationalBranchRootSchemaAuthority) -> bool {
        match self {
            Self::Original(original) => {
                let envelope = original.envelope();
                schema.matches(&envelope.schema_authority)
                    && schema.descriptor_semantics_version()
                        == envelope.descriptor_semantics_version
            }
            Self::Checkpoint(commit) => std::ptr::eq(schema, commit.schema().as_ref()),
        }
    }
    pub(super) fn shares_owner(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Original(left), Self::Original(right)) => {
                Arc::ptr_eq(left.envelope(), right.envelope())
            }
            (Self::Checkpoint(left), Self::Checkpoint(right)) => left.same_owner(right),
            (Self::Original(_), Self::Checkpoint(_)) | (Self::Checkpoint(_), Self::Original(_)) => {
                false
            }
        }
    }
}

impl super::RelationalBranchRoot {
    pub fn query_schema_context(
        &self,
    ) -> (
        crate::schema::data::SchemaVersionId,
        crate::schema::data::DescriptorSemanticsVersion,
        QueryPlanEvidenceBasis,
    ) {
        let evidence_basis = match self.commit_evidence() {
            None => QueryPlanEvidenceBasis::GenesisRuntimeBootstrap,
            Some(RelationalRootCommitEvidence::Original(envelope)) => {
                QueryPlanEvidenceBasis::CanonicalCommitEnvelope {
                    commit_id: envelope.envelope().commit.commit_id,
                }
            }
            Some(RelationalRootCommitEvidence::Checkpoint(commit)) => {
                QueryPlanEvidenceBasis::CheckpointAdmissionImage {
                    commit_id: commit.index().commit.commit_id,
                }
            }
        };
        (
            self.schema_authority().schema_version(),
            self.schema_authority().descriptor_semantics_version(),
            evidence_basis,
        )
    }
}
