mod retained_charge;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::data::aspect::AspectMask;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

use super::{
    BranchMergeRequestScopeFamily, LoweredFoundationalMergeRequest, PlannedMergeCandidateSet,
    SignalSelectedAspectRequestEntry, StructuralMergeJournalSlice,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScopedMergeCandidateBreadthSummary {
    pub boundary_candidate_width: u64,
    pub requested_scope_width: u64,
    pub admitted_candidate_width: u64,
    pub skipped_scope_width: u64,
    pub no_op_scope_width: u64,
    pub support_closure_width: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredScopedMergeCandidateSet {
    scope_family: BranchMergeRequestScopeFamily,
    boundary_candidate_nodes: Vec<NodeId>,
    requested_nodes: Vec<NodeId>,
    requested_aspects: Vec<SignalSelectedAspectRequestEntry>,
    skipped_nodes: Vec<NodeId>,
    skipped_aspects: Vec<SignalSelectedAspectRequestEntry>,
    no_op_nodes: Vec<NodeId>,
    no_op_aspects: Vec<SignalSelectedAspectRequestEntry>,
    planned_candidates: PlannedMergeCandidateSet,
    support_closure_nodes: Vec<NodeId>,
    breadth_summary: ScopedMergeCandidateBreadthSummary,
}

impl LoweredScopedMergeCandidateSet {
    pub fn lower(
        request: &LoweredFoundationalMergeRequest,
        source_journal: &StructuralMergeJournalSlice,
        source_graph: &SignalGraph,
    ) -> Result<Self, SignalError> {
        let boundary_candidate_nodes = source_journal.candidate_nodes();
        Ok(
            match request.normalized_request().normalized_scope().family() {
                BranchMergeRequestScopeFamily::FullBranch => {
                    let requested_scope_width = boundary_candidate_nodes.len() as u64;
                    let planned_candidates = PlannedMergeCandidateSet {
                        nodes: boundary_candidate_nodes.clone(),
                    };
                    Self {
                        scope_family: BranchMergeRequestScopeFamily::FullBranch,
                        boundary_candidate_nodes,
                        requested_nodes: planned_candidates.nodes.clone(),
                        requested_aspects: Vec::new(),
                        skipped_nodes: Vec::new(),
                        skipped_aspects: Vec::new(),
                        no_op_nodes: Vec::new(),
                        no_op_aspects: Vec::new(),
                        planned_candidates,
                        support_closure_nodes: Vec::new(),
                        breadth_summary: ScopedMergeCandidateBreadthSummary {
                            boundary_candidate_width: requested_scope_width,
                            requested_scope_width,
                            admitted_candidate_width: requested_scope_width,
                            skipped_scope_width: 0,
                            no_op_scope_width: 0,
                            support_closure_width: 0,
                        },
                    }
                }
                BranchMergeRequestScopeFamily::SelectedNodes => {
                    let mut admitted_nodes = Vec::new();
                    let mut skipped_nodes = Vec::new();
                    let requested_nodes = request
                        .normalized_request()
                        .normalized_scope()
                        .selected_nodes()
                        .to_vec();
                    for node in &requested_nodes {
                        if source_journal.contains_node(*node) {
                            admitted_nodes.push(*node);
                        } else {
                            skipped_nodes.push(*node);
                        }
                    }
                    let admitted_candidate_width = admitted_nodes.len() as u64;
                    let requested_scope_width = requested_nodes.len() as u64;
                    let planned_candidates = PlannedMergeCandidateSet {
                        nodes: admitted_nodes.clone(),
                    };
                    Self {
                        scope_family: BranchMergeRequestScopeFamily::SelectedNodes,
                        boundary_candidate_nodes,
                        requested_nodes,
                        requested_aspects: Vec::new(),
                        skipped_nodes,
                        skipped_aspects: Vec::new(),
                        no_op_nodes: Vec::new(),
                        no_op_aspects: Vec::new(),
                        planned_candidates,
                        support_closure_nodes: Vec::new(),
                        breadth_summary: ScopedMergeCandidateBreadthSummary {
                            boundary_candidate_width: source_journal.breadth(),
                            requested_scope_width,
                            admitted_candidate_width,
                            skipped_scope_width: requested_scope_width - admitted_candidate_width,
                            no_op_scope_width: 0,
                            support_closure_width: 0,
                        },
                    }
                }
                BranchMergeRequestScopeFamily::SelectedAspects => {
                    let requested_aspects = request
                        .normalized_request()
                        .normalized_scope()
                        .selected_aspects()
                        .to_vec();
                    let mut admitted_nodes = BTreeSet::new();
                    let mut skipped_aspects = Vec::new();
                    let mut no_op_aspects = Vec::new();
                    for entry in &requested_aspects {
                        if !source_journal.contains_node(entry.node()) {
                            skipped_aspects.push(entry.clone());
                            continue;
                        }
                        if node_contract_supports_selected_aspect(
                            source_graph,
                            entry.node(),
                            entry.aspect(),
                        )? {
                            admitted_nodes.insert(entry.node());
                        } else {
                            no_op_aspects.push(entry.clone());
                        }
                    }
                    let admitted_candidate_nodes = admitted_nodes.into_iter().collect::<Vec<_>>();
                    let requested_scope_width = requested_aspects.len() as u64;
                    let admitted_candidate_width = admitted_candidate_nodes.len() as u64;
                    let skipped_scope_width = skipped_aspects.len() as u64;
                    let no_op_scope_width = no_op_aspects.len() as u64;
                    let planned_candidates = PlannedMergeCandidateSet {
                        nodes: admitted_candidate_nodes,
                    };
                    Self {
                        scope_family: BranchMergeRequestScopeFamily::SelectedAspects,
                        boundary_candidate_nodes,
                        requested_nodes: Vec::new(),
                        requested_aspects,
                        skipped_nodes: Vec::new(),
                        skipped_aspects,
                        no_op_nodes: Vec::new(),
                        no_op_aspects,
                        planned_candidates,
                        support_closure_nodes: Vec::new(),
                        breadth_summary: ScopedMergeCandidateBreadthSummary {
                            boundary_candidate_width: source_journal.breadth(),
                            requested_scope_width,
                            admitted_candidate_width,
                            skipped_scope_width,
                            no_op_scope_width,
                            support_closure_width: 0,
                        },
                    }
                }
            },
        )
    }

    pub fn with_support_closure_nodes(
        mut self,
        support_closure_nodes: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        let support_closure_nodes = support_closure_nodes.into_iter().collect::<BTreeSet<_>>();
        self.support_closure_nodes = support_closure_nodes.into_iter().collect();
        self.breadth_summary.support_closure_width = self.support_closure_nodes.len() as u64;
        self
    }

    pub fn scope_family(&self) -> BranchMergeRequestScopeFamily {
        self.scope_family
    }

    pub fn boundary_candidate_nodes(&self) -> &[NodeId] {
        &self.boundary_candidate_nodes
    }

    pub fn requested_nodes(&self) -> &[NodeId] {
        &self.requested_nodes
    }

    pub fn requested_aspects(&self) -> &[SignalSelectedAspectRequestEntry] {
        &self.requested_aspects
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

    pub fn planned_candidates(&self) -> &PlannedMergeCandidateSet {
        &self.planned_candidates
    }

    pub fn admitted_candidate_nodes(&self) -> &[NodeId] {
        &self.planned_candidates.nodes
    }

    pub fn support_closure_nodes(&self) -> &[NodeId] {
        &self.support_closure_nodes
    }

    pub fn breadth_summary(&self) -> &ScopedMergeCandidateBreadthSummary {
        &self.breadth_summary
    }
}

fn node_contract_supports_selected_aspect(
    source_graph: &SignalGraph,
    node: NodeId,
    aspect: crate::data::aspect::Aspect,
) -> Result<bool, SignalError> {
    let contract = &source_graph.node_eval_config(node)?.contract;
    let aspect_mask = AspectMask::from(aspect);
    Ok(contract.semantics.produces.contains(aspect_mask)
        || contract.semantics.reads.contains(aspect_mask))
}
