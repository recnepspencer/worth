//! Business Back is an explicit guarded route, not an inferred history pop.
use super::{
    denial,
    index::{IndexedConnection, ValidationGraph},
    ApplicationWorkflowValidationDenial as Denial, ApplicationWorkflowValidationDenialKind as Kind,
    ValidationWorkMeter,
};
use crate::application_program::workflow::{
    ApplicationWorkflowNodeKind as Node, ApplicationWorkflowOperationRef,
};

pub(super) struct BackActionValidation {
    sources: std::collections::BTreeSet<usize>,
    visited: Vec<bool>,
    pending: Vec<usize>,
    effects: usize,
    maximum_effects: usize,
}
impl BackActionValidation {
    pub(super) fn new(effects: usize, maximum_effects: u16) -> Self {
        Self {
            sources: Default::default(),
            visited: Vec::new(),
            pending: Vec::new(),
            effects,
            maximum_effects: usize::from(maximum_effects),
        }
    }
    pub(super) fn validate(
        &mut self,
        graph: &ValidationGraph<'_>,
        edge: &IndexedConnection<'_>,
        operation: &ApplicationWorkflowOperationRef,
        work: &mut ValidationWorkMeter,
    ) -> Result<(), Denial> {
        self.effects += 1;
        if self.effects > self.maximum_effects {
            return Err(denial(Kind::EffectLimitExceeded, self.effects.to_string()));
        }
        if !self.sources.insert(edge.source) {
            return Err(denial(
                Kind::AmbiguousBackAction,
                edge.connection.source().as_str(),
            ));
        }
        let guarded = operation.binding().is_some_and(|binding| binding.2)
            && operation
                .workflow_execution_requirements()
                .approval()
                .is_none();
        if !guarded
            || edge.source == edge.target
            || matches!(graph.node(edge.source).kind(), Node::Terminal)
            || matches!(graph.node(edge.target).kind(), Node::Terminal)
        {
            return Err(denial(
                Kind::InvalidBackAction,
                edge.connection.source().as_str(),
            ));
        }
        // A phase return names a prior reachable point in this same forward
        // graph. It may cross performed operations, never arbitrary sibling state.
        self.visited.resize(graph.node_count(), false);
        self.visited.fill(false);
        self.pending.push(edge.target);
        self.visited[edge.target] = true;
        while let Some(current) = self.pending.pop() {
            work.visit_control_node();
            for &next in graph.control_successors(current) {
                work.visit_control_edge();
                if !self.visited[next] {
                    self.visited[next] = true;
                    self.pending.push(next);
                }
            }
        }
        if !self.visited[edge.source] {
            return Err(denial(
                Kind::InvalidBackAction,
                edge.connection.source().as_str(),
            ));
        }
        Ok(())
    }
}
