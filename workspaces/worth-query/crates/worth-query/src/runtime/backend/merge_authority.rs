use crate::memory_workspace::{WorthQuerySnapshotIdentity, WorthQueryWorkspaceError};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

/// Sealed lower-runtime evidence for one exact target/source merge basis.
///
/// Custom runtime backends may capture this proof only from an actual
/// relational runtime. Ordinary consumers receive it inside a Query-owned
/// context and cannot assemble or alter it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBackendMergeAuthority {
    target_branch: BranchId,
    source_branch: BranchId,
    target_snapshot_identity: WorthQuerySnapshotIdentity,
    target_basis_descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    source_basis_descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    target_head_commit_id: u64,
    source_head_commit_id: u64,
    merge_base_commit_id: u64,
    authority_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBackendMergeAuthority {
    pub fn capture(
        runtime: &RelationalRuntime,
        target_branch: &crate::runtime::WorthQueryAdmittedBranchName,
        source_branch: &crate::runtime::WorthQueryAdmittedBranchName,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let target_branch = BranchId(target_branch.as_str().to_string());
        let source_branch = BranchId(source_branch.as_str().to_string());
        if target_branch == source_branch {
            return Err(WorthQueryWorkspaceError::new(
                "branch merge authority requires distinct target and source branches",
            ));
        }
        let source_identity = runtime
            .branch_identity(&source_branch)
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        let target_identity = runtime
            .branch_identity(&target_branch)
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        let (_, source_observation) = runtime
            .observe_branch(&source_identity)
            .map_err(|denial| merge_basis_denial("source", denial))?;
        let (_, target_observation) = runtime
            .observe_branch(&target_identity)
            .map_err(|denial| merge_basis_denial("target", denial))?;
        let source_basis_descriptor = source_observation.descriptor().clone();
        let target_basis_descriptor = target_observation.descriptor().clone();
        let basis = runtime
            .history()
            .merge_branch_basis_for_observations(
                &source_observation.observation(),
                &target_observation.observation(),
            )
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        let target_snapshot_identity =
            crate::memory_workspace::snapshot_identity_from_admitted_basis(&target_observation)
                .ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "merge target branch has no exact current snapshot",
                    )
                })?;
        let target_head_commit_id = basis.target_head().commit_id.0;
        let source_head_commit_id = basis.source_head().commit_id.0;
        let merge_base_commit_id = basis.merge_base().commit().commit_id.0;
        let authority_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "backend_merge_authority_v2",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("target_branch"),
                    &target_branch.0,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("source_branch"),
                    &source_branch.0,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("target_head_commit"),
                    target_head_commit_id as usize,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("source_head_commit"),
                    source_head_commit_id as usize,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("merge_base_commit"),
                    merge_base_commit_id as usize,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("target_snapshot"),
                    &target_snapshot_identity.evidence_identity()?,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("target_basis"),
                    format!("{target_basis_descriptor:?}"),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("source_basis"),
                    format!("{source_basis_descriptor:?}"),
                )
                .seal()?;
        Ok(Self {
            target_branch,
            source_branch,
            target_snapshot_identity,
            target_basis_descriptor,
            source_basis_descriptor,
            target_head_commit_id,
            source_head_commit_id,
            merge_base_commit_id,
            authority_identity,
        })
    }

    pub fn target_branch(&self) -> &BranchId {
        &self.target_branch
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    pub fn authority_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_identity
    }

    pub(crate) fn target_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.target_snapshot_identity
    }

    pub(crate) fn validate_against(
        &self,
        runtime: &RelationalRuntime,
    ) -> Result<(), WorthQueryWorkspaceError> {
        let target_identity = runtime
            .branch_identity(&self.target_branch)
            .map_err(|_| WorthQueryWorkspaceError::new("merge target branch is missing"))?;
        let source_identity = runtime
            .branch_identity(&self.source_branch)
            .map_err(|_| WorthQueryWorkspaceError::new("merge source branch is missing"))?;
        let (_, target_basis) = runtime
            .observe_branch(&target_identity)
            .map_err(|denial| merge_basis_denial("target", denial))?;
        let (_, source_basis) = runtime
            .observe_branch(&source_identity)
            .map_err(|denial| merge_basis_denial("source", denial))?;
        if target_basis.descriptor() != &self.target_basis_descriptor
            || source_basis.descriptor() != &self.source_basis_descriptor
        {
            return Err(WorthQueryWorkspaceError::new(
                "branch merge authority is stale against the current target/source basis axes",
            ));
        }
        Ok(())
    }
}

fn merge_basis_denial(
    role: &str,
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryWorkspaceError {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            crate::memory_workspace::WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            crate::memory_workspace::WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            crate::memory_workspace::WorthQueryWorkspaceErrorKind::SnapshotIdentityExhausted
        }
        _ => crate::memory_workspace::WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
    };
    WorthQueryWorkspaceError::with_kind(
        kind,
        format!("merge {role} current basis observation failed: {denial:?}"),
    )
}
