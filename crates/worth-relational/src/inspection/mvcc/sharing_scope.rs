use std::collections::BTreeSet;
use std::sync::Arc;

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};
use crate::history::data::BranchId;
use crate::history::RelationalCommitArtifact;
use crate::runtime::RelationalRuntime;

use super::RelationalBranchSharingInspectionDenial;

pub(super) struct BranchSharingScopeEntry {
    pub(super) branch_id: BranchId,
    pub(super) root: Arc<RelationalBranchRoot>,
    /// The branch root's immutable catalog artifact, carried by shared
    /// ownership rather than borrowed out of the history ledger.
    pub(super) artifact: Arc<RelationalCommitArtifact>,
    pub(super) coordination_cell: crate::branch::RelationalBranchCoordinationCellId,
    pub(super) coordination_contacts: u64,
    pub(super) coordination_waits: u64,
}

pub(super) fn resolve_sharing_scope(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branches: &[RelationalBranchIdentity],
) -> Result<Vec<BranchSharingScopeEntry>, RelationalBranchSharingInspectionDenial> {
    let mut seen_branches = BTreeSet::new();
    let mut verified_roots = BTreeSet::new();
    let mut scope = Vec::with_capacity(branches.len());
    for identity in branches {
        validate_runtime_and_uniqueness(runtime, identity, &mut seen_branches)?;
        let cell = runtime
            .history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .ok_or(RelationalBranchSharingInspectionDenial::UnknownBranch)?;
        let root = cell
            .root()
            .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
        let commit_id = root
            .commit_id()
            .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
        let artifact = runtime
            .history
            .commit_artifact(commit_id)
            .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
        if verified_roots.insert(root.id())
            && (!runtime
                .services
                .symbols
                .with_read(|symbols| root.is_complete(execution, symbols))
                || !artifact.links_root(&root)
                || root.axes().is_none())
        {
            return Err(RelationalBranchSharingInspectionDenial::RootUnavailable);
        }
        scope.push(BranchSharingScopeEntry {
            branch_id: identity.branch_id().clone(),
            root,
            artifact,
            coordination_cell: cell.coordination().id(),
            coordination_contacts: cell.coordination().contact_count(),
            coordination_waits: cell.coordination().wait_count(),
        });
    }
    Ok(scope)
}

fn validate_runtime_and_uniqueness(
    runtime: &RelationalRuntime,
    identity: &RelationalBranchIdentity,
    seen_branches: &mut BTreeSet<RelationalBranchIdentity>,
) -> Result<(), RelationalBranchSharingInspectionDenial> {
    if identity.runtime_instance_id() != runtime.runtime_instance_id() {
        return Err(RelationalBranchSharingInspectionDenial::ForeignRuntime);
    }
    if !seen_branches.insert(identity.clone()) {
        return Err(RelationalBranchSharingInspectionDenial::DuplicateBranch);
    }
    Ok(())
}
