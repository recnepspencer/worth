mod retained_charge;

use serde::{Deserialize, Serialize};

use super::proof::canonical_digest;
use super::{
    BranchMergeRequestScopeFamily, LoweredFoundationalMergeRequest, LoweredScopedMergeCandidateSet,
    ScopedMergeCandidateBreadthSummary, SignalSelectedAspectRequestEntry,
};
use crate::data::handle::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedMergeProofPacket {
    scope_family: BranchMergeRequestScopeFamily,
    declaration_digest: String,
    admitted_scope_digest: String,
    skipped_scope_digest: Option<String>,
    no_op_scope_digest: Option<String>,
    breadth_summary: ScopedMergeCandidateBreadthSummary,
    requested_nodes: Vec<NodeId>,
    requested_aspects: Vec<SignalSelectedAspectRequestEntry>,
    admitted_nodes: Vec<NodeId>,
    admitted_aspects: Vec<SignalSelectedAspectRequestEntry>,
    skipped_nodes: Vec<NodeId>,
    skipped_aspects: Vec<SignalSelectedAspectRequestEntry>,
    no_op_nodes: Vec<NodeId>,
    no_op_aspects: Vec<SignalSelectedAspectRequestEntry>,
    support_closure_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize)]
struct ScopeDigestBasis<'a> {
    scope_family: BranchMergeRequestScopeFamily,
    nodes: &'a [NodeId],
    aspects: &'a [SignalSelectedAspectRequestEntry],
}

impl ScopedMergeProofPacket {
    pub fn from_request_and_candidates(
        request: &LoweredFoundationalMergeRequest,
        scoped_candidates: &LoweredScopedMergeCandidateSet,
    ) -> Self {
        let normalized_scope = request.normalized_request().normalized_scope();
        let requested_nodes = scoped_candidates.requested_nodes().to_vec();
        let requested_aspects = scoped_candidates.requested_aspects().to_vec();
        let admitted_nodes = scoped_candidates.admitted_candidate_nodes().to_vec();
        let admitted_aspects = admitted_aspects(scoped_candidates);
        let skipped_nodes = scoped_candidates.skipped_nodes().to_vec();
        let skipped_aspects = scoped_candidates.skipped_aspects().to_vec();
        let no_op_nodes = scoped_candidates.no_op_nodes().to_vec();
        let no_op_aspects = scoped_candidates.no_op_aspects().to_vec();
        let support_closure_nodes = scoped_candidates.support_closure_nodes().to_vec();
        let scope_family = normalized_scope.family();
        let declaration_digest = normalized_scope.scope_digest().to_owned();
        let admitted_scope_digest = admitted_scope_digest(
            scope_family,
            &declaration_digest,
            &admitted_nodes,
            &admitted_aspects,
        );
        let skipped_scope_digest =
            optional_scope_digest(scope_family, &skipped_nodes, &skipped_aspects);
        let no_op_scope_digest = optional_scope_digest(scope_family, &no_op_nodes, &no_op_aspects);
        Self {
            scope_family,
            declaration_digest,
            admitted_scope_digest,
            skipped_scope_digest,
            no_op_scope_digest,
            breadth_summary: scoped_candidates.breadth_summary().clone(),
            requested_nodes,
            requested_aspects,
            admitted_nodes,
            admitted_aspects,
            skipped_nodes,
            skipped_aspects,
            no_op_nodes,
            no_op_aspects,
            support_closure_nodes,
        }
    }

    pub fn scope_family(&self) -> BranchMergeRequestScopeFamily {
        self.scope_family
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn admitted_scope_digest(&self) -> &str {
        &self.admitted_scope_digest
    }

    pub fn skipped_scope_digest(&self) -> Option<&str> {
        self.skipped_scope_digest.as_deref()
    }

    pub fn no_op_scope_digest(&self) -> Option<&str> {
        self.no_op_scope_digest.as_deref()
    }

    pub fn breadth_summary(&self) -> &ScopedMergeCandidateBreadthSummary {
        &self.breadth_summary
    }

    pub fn requested_nodes(&self) -> &[NodeId] {
        &self.requested_nodes
    }

    pub fn requested_aspects(&self) -> &[SignalSelectedAspectRequestEntry] {
        &self.requested_aspects
    }

    pub fn admitted_nodes(&self) -> &[NodeId] {
        &self.admitted_nodes
    }

    pub fn admitted_aspects(&self) -> &[SignalSelectedAspectRequestEntry] {
        &self.admitted_aspects
    }

    pub fn skipped_nodes(&self) -> &[NodeId] {
        &self.skipped_nodes
    }

    pub fn skipped_aspects(&self) -> &[SignalSelectedAspectRequestEntry] {
        &self.skipped_aspects
    }

    pub fn no_op_nodes(&self) -> &[NodeId] {
        &self.no_op_nodes
    }

    pub fn no_op_aspects(&self) -> &[SignalSelectedAspectRequestEntry] {
        &self.no_op_aspects
    }

    pub fn support_closure_nodes(&self) -> &[NodeId] {
        &self.support_closure_nodes
    }
}

fn admitted_aspects(
    scoped_candidates: &LoweredScopedMergeCandidateSet,
) -> Vec<SignalSelectedAspectRequestEntry> {
    scoped_candidates
        .requested_aspects()
        .iter()
        .filter(|entry| {
            !scoped_candidates.skipped_aspects().contains(entry)
                && !scoped_candidates.no_op_aspects().contains(entry)
        })
        .cloned()
        .collect()
}

fn admitted_scope_digest(
    scope_family: BranchMergeRequestScopeFamily,
    declaration_digest: &str,
    nodes: &[NodeId],
    aspects: &[SignalSelectedAspectRequestEntry],
) -> String {
    match scope_family {
        BranchMergeRequestScopeFamily::FullBranch => declaration_digest.to_owned(),
        BranchMergeRequestScopeFamily::SelectedNodes
        | BranchMergeRequestScopeFamily::SelectedAspects => {
            canonical_scope_digest(scope_family, nodes, aspects)
        }
    }
}

fn optional_scope_digest(
    scope_family: BranchMergeRequestScopeFamily,
    nodes: &[NodeId],
    aspects: &[SignalSelectedAspectRequestEntry],
) -> Option<String> {
    if nodes.is_empty() && aspects.is_empty() {
        None
    } else {
        Some(canonical_scope_digest(scope_family, nodes, aspects))
    }
}

fn canonical_scope_digest(
    scope_family: BranchMergeRequestScopeFamily,
    nodes: &[NodeId],
    aspects: &[SignalSelectedAspectRequestEntry],
) -> String {
    canonical_digest(&ScopeDigestBasis {
        scope_family,
        nodes,
        aspects,
    })
}
