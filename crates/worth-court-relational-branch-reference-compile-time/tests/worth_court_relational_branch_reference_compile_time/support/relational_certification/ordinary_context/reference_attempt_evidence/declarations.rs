pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity, worth_relational::facade::branch::RelationalBranchIdentityDenial, worth_relational::facade::branch::RelationalBranchReferenceState, worth_relational::facade::branch::RelationalForkSourceDescriptor};
pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::history::CommitId, worth_relational::facade::history::RelationalCommitIdentity};
pub(crate) use {worth_relational::facade::runtime::RelationalPhase4ReferenceCostCounters, worth_relational::facade::runtime::RelationalRuntime};

/// Public owner observations and immutable catalog identity captured around a
/// Phase-4 fork attempt. This is evidence, not an authority carrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Phase4ReferenceEvidence {
    pub(crate) source: Option<RelationalForkSourceDescriptor>,
    pub(crate) target: Option<RelationalForkSourceDescriptor>,
    pub(crate) source_identity: Result<RelationalBranchIdentity, RelationalBranchIdentityDenial>,
    pub(crate) target_identity: Result<RelationalBranchIdentity, RelationalBranchIdentityDenial>,
    pub(crate) source_state: Option<RelationalBranchReferenceState>,
    pub(crate) target_state: Option<RelationalBranchReferenceState>,
    pub(crate) catalog_count: usize,
    pub(crate) artifact_identity: Option<RelationalCommitIdentity>,
    pub(crate) artifact_parents: Option<Vec<CommitId>>,
    pub(crate) counters: RelationalPhase4ReferenceCostCounters,
}

pub(crate) fn capture_reference_evidence(
    runtime: &RelationalRuntime,
    source_branch: &BranchId,
    target_branch: &BranchId,
    commit_id: CommitId,
) -> Phase4ReferenceEvidence {
    Phase4ReferenceEvidence {
        source: observe_descriptor(runtime, source_branch),
        target: observe_descriptor(runtime, target_branch),
        source_identity: runtime.branch_identity(source_branch),
        target_identity: runtime.branch_identity(target_branch),
        source_state: runtime.branch_reference_state(source_branch),
        target_state: runtime.branch_reference_state(target_branch),
        catalog_count: runtime.history().immutable_commit_count(),
        artifact_identity: runtime.history().immutable_commit_identity(commit_id),
        artifact_parents: runtime
            .history()
            .immutable_commit_receipt(commit_id)
            .map(|receipt| receipt.parents),
        counters: runtime.phase4_reference_cost_counters(),
    }
}

pub(crate) fn assert_denial_left_no_reference_residue(
    before: &Phase4ReferenceEvidence,
    after: &Phase4ReferenceEvidence,
) {
    assert_eq!(
        after.source, before.source,
        "source reference moved on denial"
    );
    assert_eq!(
        after.target, before.target,
        "target reference moved on denial"
    );
    assert_eq!(
        after.source_identity, before.source_identity,
        "source identity changed on denial"
    );
    assert_eq!(
        after.target_identity, before.target_identity,
        "target identity changed on denial"
    );
    assert_eq!(
        after.source_state, before.source_state,
        "source branch-cell checkpoint changed on denial"
    );
    assert_eq!(
        after.target_state, before.target_state,
        "target branch-cell checkpoint changed on denial"
    );
    assert_eq!(
        after.catalog_count, before.catalog_count,
        "catalog length changed on denial"
    );
    assert_eq!(
        after.artifact_identity, before.artifact_identity,
        "canonical artifact identity changed on denial"
    );
    assert_eq!(
        after.artifact_parents, before.artifact_parents,
        "canonical ordered parentage changed on denial"
    );
    assert_eq!(
        after.counters.artifact_clones, before.counters.artifact_clones,
        "artifact materialization changed on denial"
    );
}

pub(crate) fn observe_descriptor(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) -> Option<RelationalForkSourceDescriptor> {
    runtime
        .observe_fork_source(branch_id)
        .ok()
        .map(|(descriptor, _basis)| descriptor)
}