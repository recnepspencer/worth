use std::mem::size_of;

use crate::application_program::workflow::{
    ApplicationWorkflowAssessmentApplicability, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowDataFlow, ApplicationWorkflowNodeKind,
};

use super::{
    control::ControlProof,
    denial,
    index::{NodeIndex, ValidationGraph},
    ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind,
    ValidationWorkMeter,
};

pub(super) fn validate_availability(
    start: NodeIndex,
    graph: &ValidationGraph<'_>,
    control: &ControlProof,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let has_data = graph.connections().iter().any(|indexed| {
        work.visit_dominance_candidate();
        matches!(
            indexed.connection.kind_ref(),
            ApplicationWorkflowConnectionKind::Data(_)
        )
    });
    if !has_data {
        return Ok(());
    }
    let dominance = DominanceIndex::build(start, graph, &control.topological_order, work);
    for indexed in graph.connections() {
        if !matches!(
            indexed.connection.kind_ref(),
            ApplicationWorkflowConnectionKind::Data(_)
        ) {
            continue;
        }
        work.visit_dominance_query();
        let conditional_assessment_evidence = matches!(
            indexed.connection.kind_ref(),
            ApplicationWorkflowConnectionKind::Data(
                ApplicationWorkflowDataFlow::AssessmentEvidence
            )
        ) && matches!(
            graph.node(indexed.source).kind(),
            ApplicationWorkflowNodeKind::Assessment(assessment)
                if !matches!(assessment.applicability(), ApplicationWorkflowAssessmentApplicability::Always)
        );
        // Conditional evidence is independently collectible off-cursor. Its
        // exact authored identity, applicability, and source currentness are
        // checked when the typed join consumes it; control ancestry is not
        // the availability authority for this one data-flow kind. Control
        // validation has already made both endpoints reachable from start.
        let available = conditional_assessment_evidence
            || dominance.dominates(indexed.source, indexed.target, work);
        if indexed.source == indexed.target || !available {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::UnavailableDataFlow,
                format!(
                    "{} -> {}",
                    indexed.connection.source().as_str(),
                    indexed.connection.target().as_str()
                ),
            ));
        }
    }
    work.observe_index_bytes(
        graph
            .retained_bytes()
            .saturating_add(dominance.retained_bytes())
            .saturating_add(control.topological_order.capacity() * size_of::<NodeIndex>()),
    );
    Ok(())
}

struct DominanceIndex {
    node_count: usize,
    levels: usize,
    depth: Vec<usize>,
    ancestors: Vec<NodeIndex>,
}

impl DominanceIndex {
    fn build(
        start: NodeIndex,
        graph: &ValidationGraph<'_>,
        topological_order: &[NodeIndex],
        work: &mut ValidationWorkMeter,
    ) -> Self {
        let node_count = graph.node_count();
        let levels = logarithmic_levels(node_count);
        let mut index = Self {
            node_count,
            levels,
            depth: vec![0; node_count],
            ancestors: vec![start; node_count.saturating_mul(levels)],
        };
        for &node in topological_order {
            if node == start {
                continue;
            }
            let mut predecessors = graph.control_predecessors(node).iter().copied();
            let mut immediate = predecessors
                .next()
                .expect("every reachable non-start node has a predecessor");
            work.visit_dominance_predecessor();
            for predecessor in predecessors {
                work.visit_dominance_predecessor();
                immediate = index.lowest_common_ancestor(immediate, predecessor, work);
            }
            index.depth[node] = index.depth[immediate].saturating_add(1);
            index.set_ancestor(0, node, immediate);
            for level in 1..levels {
                let lower = index.ancestor(level - 1, node);
                let ancestor = index.ancestor(level - 1, lower);
                index.set_ancestor(level, node, ancestor);
                work.write_dominance_table();
            }
        }
        index
    }

    fn dominates(
        &self,
        candidate: NodeIndex,
        node: NodeIndex,
        work: &mut ValidationWorkMeter,
    ) -> bool {
        self.lowest_common_ancestor(candidate, node, work) == candidate
    }

    fn lowest_common_ancestor(
        &self,
        mut left: NodeIndex,
        mut right: NodeIndex,
        work: &mut ValidationWorkMeter,
    ) -> NodeIndex {
        if self.depth[left] < self.depth[right] {
            std::mem::swap(&mut left, &mut right);
        }
        let difference = self.depth[left] - self.depth[right];
        for level in (0..self.levels).rev() {
            if difference & (1_usize << level) != 0 {
                left = self.ancestor(level, left);
                work.visit_dominance_lift();
            }
        }
        if left == right {
            return left;
        }
        for level in (0..self.levels).rev() {
            let left_ancestor = self.ancestor(level, left);
            let right_ancestor = self.ancestor(level, right);
            if left_ancestor != right_ancestor {
                left = left_ancestor;
                right = right_ancestor;
                work.visit_dominance_lift();
                work.visit_dominance_lift();
            }
        }
        self.ancestor(0, left)
    }

    fn ancestor(&self, level: usize, node: NodeIndex) -> NodeIndex {
        self.ancestors[level * self.node_count + node]
    }

    fn set_ancestor(&mut self, level: usize, node: NodeIndex, ancestor: NodeIndex) {
        self.ancestors[level * self.node_count + node] = ancestor;
    }

    fn retained_bytes(&self) -> usize {
        self.depth
            .capacity()
            .saturating_mul(size_of::<usize>())
            .saturating_add(
                self.ancestors
                    .capacity()
                    .saturating_mul(size_of::<NodeIndex>()),
            )
    }
}

fn logarithmic_levels(node_count: usize) -> usize {
    if node_count <= 1 {
        1
    } else {
        (usize::BITS - (node_count - 1).leading_zeros()) as usize
    }
}
