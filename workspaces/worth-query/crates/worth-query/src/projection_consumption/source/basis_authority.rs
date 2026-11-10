use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::query_context::QueryContextExecutionFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionSourceBasisAuthority {
    pub(super) kind: ProjectionSourceBasisAuthorityKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProjectionSourceBasisAuthorityKind {
    RuntimeSnapshot(WorthQuerySnapshotIdentity),
    QueryContext {
        family: QueryContextExecutionFamily,
        basis_digest: String,
    },
    Certification(WorthQueryEvidenceIdentity),
}

impl ProjectionSourceBasisAuthority {
    pub fn canonical_digest(&self) -> Option<worth_foundational::facade::CanonicalDerivedDigest> {
        match &self.kind {
            ProjectionSourceBasisAuthorityKind::RuntimeSnapshot(identity) => {
                Some(identity.evidence_identity().canonical_digest().clone())
            }
            ProjectionSourceBasisAuthorityKind::Certification(identity) => {
                Some(identity.canonical_digest().clone())
            }
            ProjectionSourceBasisAuthorityKind::QueryContext { .. } => None,
        }
    }

    pub fn snapshot_identity(&self) -> Option<&WorthQuerySnapshotIdentity> {
        match &self.kind {
            ProjectionSourceBasisAuthorityKind::RuntimeSnapshot(identity) => Some(identity),
            ProjectionSourceBasisAuthorityKind::QueryContext { .. }
            | ProjectionSourceBasisAuthorityKind::Certification(_) => None,
        }
    }

    pub fn query_context_family(&self) -> Option<&QueryContextExecutionFamily> {
        match &self.kind {
            ProjectionSourceBasisAuthorityKind::QueryContext { family, .. } => Some(family),
            ProjectionSourceBasisAuthorityKind::RuntimeSnapshot(_)
            | ProjectionSourceBasisAuthorityKind::Certification(_) => None,
        }
    }

    pub fn terminal_projection_for_reporting(&self) -> String {
        match &self.kind {
            ProjectionSourceBasisAuthorityKind::RuntimeSnapshot(identity) => {
                identity.evidence_identity().as_str().to_string()
            }
            ProjectionSourceBasisAuthorityKind::QueryContext { basis_digest, .. } => {
                basis_digest.clone()
            }
            ProjectionSourceBasisAuthorityKind::Certification(identity) => {
                identity.as_str().to_string()
            }
        }
    }

    pub(crate) fn runtime_snapshot(identity: WorthQuerySnapshotIdentity) -> Self {
        Self {
            kind: ProjectionSourceBasisAuthorityKind::RuntimeSnapshot(identity),
        }
    }

    pub(crate) fn query_context(
        family: QueryContextExecutionFamily,
        basis_digest: impl Into<String>,
    ) -> Self {
        Self {
            kind: ProjectionSourceBasisAuthorityKind::QueryContext {
                family,
                basis_digest: basis_digest.into(),
            },
        }
    }

    pub(crate) fn certification(identity: WorthQueryEvidenceIdentity) -> Self {
        Self {
            kind: ProjectionSourceBasisAuthorityKind::Certification(identity),
        }
    }
}
