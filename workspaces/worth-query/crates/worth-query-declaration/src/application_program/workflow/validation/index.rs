use std::mem::size_of;

use crate::application_program::workflow::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow, ApplicationWorkflowNode,
    ApplicationWorkflowNodeIdentity,
};

use super::{
    denial, ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind,
    ValidationWorkMeter,
};

pub(super) type NodeIndex = usize;

pub(super) struct IndexedConnection<'a> {
    pub(super) connection: &'a ApplicationWorkflowConnection,
    pub(super) source: NodeIndex,
    pub(super) target: NodeIndex,
}

#[derive(Clone, Default)]
pub(super) struct NodeConnectionSummary {
    incoming_data: [u32; 7],
    outgoing_control: [u32; 8],
    retry_count: u32,
    retry_trigger: Option<ApplicationWorkflowControlOutcome>,
}

impl NodeConnectionSummary {
    pub(super) fn incoming(&self, flow: ApplicationWorkflowDataFlow) -> usize {
        self.incoming_data[data_index(flow)] as usize
    }

    pub(super) fn outcome(&self, outcome: ApplicationWorkflowControlOutcome) -> usize {
        self.outgoing_control[control_index(outcome)] as usize
    }

    pub(super) const fn retry_count(&self) -> usize {
        self.retry_count as usize
    }

    pub(super) const fn retry_trigger(&self) -> Option<ApplicationWorkflowControlOutcome> {
        self.retry_trigger
    }

    pub(super) fn has_any_control_successor(&self) -> bool {
        self.retry_count != 0 || self.outgoing_control.iter().any(|count| *count != 0)
    }
}

pub(super) struct ValidationGraph<'a> {
    nodes: Vec<&'a ApplicationWorkflowNode>,
    connections: Vec<IndexedConnection<'a>>,
    summaries: Vec<NodeConnectionSummary>,
    control_successors: DirectedCsr,
    control_predecessors: DirectedCsr,
}

impl<'a> ValidationGraph<'a> {
    pub(super) fn build(
        nodes: &'a [ApplicationWorkflowNode],
        connections: &'a [ApplicationWorkflowConnection],
        work: &mut ValidationWorkMeter,
    ) -> Result<Self, ApplicationWorkflowValidationDenial> {
        let indexed_nodes = index_nodes(nodes, work);
        let mut summaries = vec![NodeConnectionSummary::default(); nodes.len()];
        let mut control_edges = Vec::new();
        let indexed_connections = index_connections(
            &indexed_nodes,
            connections,
            &mut summaries,
            &mut control_edges,
            work,
        )?;
        let control_successors = DirectedCsr::new(nodes.len(), &control_edges, false);
        let control_predecessors = DirectedCsr::new(nodes.len(), &control_edges, true);
        let graph = Self {
            nodes: indexed_nodes,
            connections: indexed_connections,
            summaries,
            control_successors,
            control_predecessors,
        };
        work.observe_index_bytes(
            graph
                .retained_bytes()
                .saturating_add(control_edges.capacity() * size_of::<(usize, usize)>()),
        );
        Ok(graph)
    }

    pub(super) fn resolve(
        &self,
        identity: &ApplicationWorkflowNodeIdentity,
        work: &mut ValidationWorkMeter,
    ) -> Option<NodeIndex> {
        resolve(&self.nodes, identity, work)
    }

    pub(super) fn node(&self, index: NodeIndex) -> &'a ApplicationWorkflowNode {
        self.nodes[index]
    }

    pub(super) fn nodes(
        &self,
    ) -> impl Iterator<Item = (NodeIndex, &'a ApplicationWorkflowNode)> + '_ {
        self.nodes.iter().copied().enumerate()
    }

    pub(super) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(super) fn connections(&self) -> &[IndexedConnection<'a>] {
        &self.connections
    }

    pub(super) fn summary(&self, index: NodeIndex) -> &NodeConnectionSummary {
        &self.summaries[index]
    }

    pub(super) fn control_successors(&self, index: NodeIndex) -> &[NodeIndex] {
        self.control_successors.neighbors(index)
    }

    pub(super) fn control_predecessors(&self, index: NodeIndex) -> &[NodeIndex] {
        self.control_predecessors.neighbors(index)
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.nodes
            .capacity()
            .saturating_mul(size_of::<&ApplicationWorkflowNode>())
            .saturating_add(
                self.connections
                    .capacity()
                    .saturating_mul(size_of::<IndexedConnection<'_>>()),
            )
            .saturating_add(
                self.summaries
                    .capacity()
                    .saturating_mul(size_of::<NodeConnectionSummary>()),
            )
            .saturating_add(self.control_successors.retained_bytes())
            .saturating_add(self.control_predecessors.retained_bytes())
    }
}

fn index_nodes<'a>(
    nodes: &'a [ApplicationWorkflowNode],
    work: &mut ValidationWorkMeter,
) -> Vec<&'a ApplicationWorkflowNode> {
    let mut indexed = Vec::with_capacity(nodes.len());
    for node in nodes {
        work.index_node();
        indexed.push(node);
    }
    indexed.sort_unstable_by(|left, right| left.identity().cmp(right.identity()));
    indexed
}

fn index_connections<'a>(
    nodes: &[&ApplicationWorkflowNode],
    connections: &'a [ApplicationWorkflowConnection],
    summaries: &mut [NodeConnectionSummary],
    control_edges: &mut Vec<(NodeIndex, NodeIndex)>,
    work: &mut ValidationWorkMeter,
) -> Result<Vec<IndexedConnection<'a>>, ApplicationWorkflowValidationDenial> {
    let mut indexed = Vec::with_capacity(connections.len());
    for connection in connections {
        work.index_connection();
        let source = resolve(nodes, connection.source(), work)
            .ok_or_else(|| unknown_endpoint(connection))?;
        let target = resolve(nodes, connection.target(), work)
            .ok_or_else(|| unknown_endpoint(connection))?;
        match connection.kind_ref() {
            ApplicationWorkflowConnectionKind::Control(outcome) => {
                summaries[source].outgoing_control[control_index(*outcome)] += 1;
                control_edges.push((source, target));
            }
            ApplicationWorkflowConnectionKind::Data(flow) => {
                summaries[target].incoming_data[data_index(*flow)] += 1;
            }
            ApplicationWorkflowConnectionKind::Retry(retry) => {
                let summary = &mut summaries[source];
                summary.retry_count += 1;
                summary.retry_trigger.get_or_insert(retry.trigger());
            }
        }
        indexed.push(IndexedConnection {
            connection,
            source,
            target,
        });
    }
    Ok(indexed)
}

struct DirectedCsr {
    offsets: Vec<usize>,
    neighbors: Vec<NodeIndex>,
}

impl DirectedCsr {
    fn new(node_count: usize, edges: &[(NodeIndex, NodeIndex)], reverse: bool) -> Self {
        let mut offsets = vec![0_usize; node_count.saturating_add(1)];
        for &(source, target) in edges {
            let owner = if reverse { target } else { source };
            offsets[owner + 1] += 1;
        }
        for index in 1..offsets.len() {
            offsets[index] = offsets[index].saturating_add(offsets[index - 1]);
        }
        let mut cursors = offsets[..node_count].to_vec();
        let mut neighbors = vec![0_usize; edges.len()];
        for &(source, target) in edges {
            let (owner, neighbor) = if reverse {
                (target, source)
            } else {
                (source, target)
            };
            neighbors[cursors[owner]] = neighbor;
            cursors[owner] += 1;
        }
        Self { offsets, neighbors }
    }

    fn neighbors(&self, index: NodeIndex) -> &[NodeIndex] {
        &self.neighbors[self.offsets[index]..self.offsets[index + 1]]
    }

    fn retained_bytes(&self) -> usize {
        self.offsets
            .capacity()
            .saturating_mul(size_of::<usize>())
            .saturating_add(
                self.neighbors
                    .capacity()
                    .saturating_mul(size_of::<NodeIndex>()),
            )
    }
}

fn resolve(
    nodes: &[&ApplicationWorkflowNode],
    identity: &ApplicationWorkflowNodeIdentity,
    work: &mut ValidationWorkMeter,
) -> Option<NodeIndex> {
    work.lookup_identity();
    nodes
        .binary_search_by(|node| node.identity().cmp(identity))
        .ok()
}

fn unknown_endpoint(
    connection: &ApplicationWorkflowConnection,
) -> ApplicationWorkflowValidationDenial {
    denial(
        ApplicationWorkflowValidationDenialKind::UnknownConnectionEndpoint,
        connection_subject(connection),
    )
}

fn connection_subject(connection: &ApplicationWorkflowConnection) -> String {
    format!(
        "{} -> {}",
        connection.source().as_str(),
        connection.target().as_str()
    )
}

const fn data_index(flow: ApplicationWorkflowDataFlow) -> usize {
    match flow {
        ApplicationWorkflowDataFlow::ProposalSubject => 0,
        ApplicationWorkflowDataFlow::AssessmentSubject => 1,
        ApplicationWorkflowDataFlow::ConditionSubject => 2,
        ApplicationWorkflowDataFlow::AssessmentEvidence => 3,
        ApplicationWorkflowDataFlow::JoinedEvidence => 4,
        ApplicationWorkflowDataFlow::ApprovalAuthority => 5,
        ApplicationWorkflowDataFlow::OperationInput => 6,
    }
}

const fn control_index(outcome: ApplicationWorkflowControlOutcome) -> usize {
    match outcome {
        ApplicationWorkflowControlOutcome::Completed => 0,
        ApplicationWorkflowControlOutcome::Approved => 1,
        ApplicationWorkflowControlOutcome::Rejected => 2,
        ApplicationWorkflowControlOutcome::EvidenceSatisfied => 3,
        ApplicationWorkflowControlOutcome::EvidenceFailed => 4,
        ApplicationWorkflowControlOutcome::ConditionSatisfied => 5,
        ApplicationWorkflowControlOutcome::ConditionUnsatisfied => 6,
        ApplicationWorkflowControlOutcome::RetryExhausted => 7,
    }
}
