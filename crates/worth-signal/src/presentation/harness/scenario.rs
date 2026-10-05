use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::{ExecutionAuthority, LeaseRequest};

use worth_harness::facade::{ExecutionRequest, MutationBatch, ScenarioFixture, ScenarioPlan};

use crate::facade::*;

use super::runtime::{
    SignalCheckedEvaluationDriver, SignalEvaluationDriver, SignalFixtureFactory,
    SignalHarnessRuntime, SignalMutationAction,
};

pub struct SignalScenario {
    name: String,
    graph: SignalGraph,
    evaluator: Option<Arc<dyn SignalEvaluationDriver>>,
    checked_evaluator: Option<Arc<dyn SignalCheckedEvaluationDriver>>,
    execution_authority: Option<Arc<ExecutionAuthority>>,
    lease_request: Option<LeaseRequest>,
    labels: BTreeMap<String, NodeId>,
    pending_dependencies: BTreeMap<NodeId, Vec<DependencyEdge>>,
    declared_inputs: Vec<String>,
    declared_observations: Vec<String>,
    metadata: BTreeMap<String, String>,
}

impl SignalScenario {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph: SignalGraph::new(),
            evaluator: None,
            checked_evaluator: None,
            execution_authority: None,
            lease_request: None,
            labels: BTreeMap::new(),
            pending_dependencies: BTreeMap::new(),
            declared_inputs: Vec::new(),
            declared_observations: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn graph(&self) -> &SignalGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut SignalGraph {
        self.flush_pending_dependencies()
            .expect("signal scenario pending dependency batch should flush cleanly");
        &mut self.graph
    }

    pub fn label(&mut self, name: impl Into<String>, node: NodeId) -> NodeId {
        self.labels.insert(name.into(), node);
        node
    }

    pub fn node(&mut self, label: impl Into<String>) -> NodeId {
        let node = self.graph.node().build();
        self.label(label, node)
    }

    pub fn build_node(
        &mut self,
        label: impl Into<String>,
        build: impl FnOnce(&mut SignalGraph) -> NodeId,
    ) -> NodeId {
        self.flush_pending_dependencies()
            .expect("signal scenario pending dependency batch should flush cleanly");
        let node = build(&mut self.graph);
        self.label(label, node)
    }

    pub fn resolve(&self, label: &str) -> Result<NodeId, SignalError> {
        self.labels.get(label).copied().ok_or_else(|| {
            SignalError::invalid_input(format!("unknown signal scenario label `{label}`"))
        })
    }

    pub fn dependency(
        &mut self,
        downstream_label: &str,
        upstream_label: &str,
        aspect: Aspect,
    ) -> Result<&mut Self, SignalError> {
        let downstream = self.resolve(downstream_label)?;
        let upstream = self.resolve(upstream_label)?;
        let mut dependencies = self.pending_dependencies_for(downstream)?;
        dependencies.push(DependencyEdge::new(upstream, aspect));
        self.pending_dependencies.insert(downstream, dependencies);
        Ok(self)
    }

    pub fn partition_dependency(
        &mut self,
        downstream_label: &str,
        upstream_label: &str,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
    ) -> Result<&mut Self, SignalError> {
        let downstream = self.resolve(downstream_label)?;
        let upstream = self.resolve(upstream_label)?;
        let mut dependencies = self.pending_dependencies_for(downstream)?;
        dependencies.push(DependencyEdge::whole_partition(upstream, aspect, partition));
        self.pending_dependencies.insert(downstream, dependencies);
        Ok(self)
    }

    pub fn partition_detail_dependency(
        &mut self,
        downstream_label: &str,
        upstream_label: &str,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Result<&mut Self, SignalError> {
        let downstream = self.resolve(downstream_label)?;
        let upstream = self.resolve(upstream_label)?;
        let mut dependencies = self.pending_dependencies_for(downstream)?;
        dependencies.push(DependencyEdge::partition_detail(
            upstream, aspect, partition, detail,
        ));
        self.pending_dependencies.insert(downstream, dependencies);
        Ok(self)
    }

    pub fn input(mut self, label: impl Into<String>) -> Self {
        self.declared_inputs.push(label.into());
        self
    }

    pub fn observe(mut self, label: impl Into<String>) -> Self {
        self.declared_observations.push(label.into());
        self
    }

    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_evaluator<F>(mut self, evaluator: F) -> Self
    where
        F: SignalEvaluationDriver + 'static,
    {
        self.evaluator = Some(Arc::new(evaluator));
        self
    }

    pub fn set_evaluator<F>(&mut self, evaluator: F)
    where
        F: SignalEvaluationDriver + 'static,
    {
        self.evaluator = Some(Arc::new(evaluator));
    }

    pub fn set_checked_evaluator<F>(&mut self, evaluator: F)
    where
        F: SignalCheckedEvaluationDriver + 'static,
    {
        self.checked_evaluator = Some(Arc::new(evaluator));
    }

    pub fn with_execution_authority(
        mut self,
        authority: Arc<ExecutionAuthority>,
        request: LeaseRequest,
    ) -> Self {
        self.execution_authority = Some(authority);
        self.lease_request = Some(request);
        self
    }

    pub fn fixture(self) -> Result<ScenarioFixture<SignalFixtureFactory>, SignalError> {
        let mut this = self;
        this.flush_pending_dependencies()?;
        let evaluator = this.evaluator.ok_or_else(|| {
            SignalError::invalid_input("signal scenario requires an evaluator before compile")
        })?;
        let checked_evaluator = this.checked_evaluator;
        let execution_authority = this.execution_authority;
        let lease_request = this.lease_request;
        let name = this.name;
        let graph = this.graph;
        let labels = this.labels;
        let declared_inputs = this.declared_inputs;
        let declared_observations = this.declared_observations;
        let metadata = this.metadata;

        let fixture = SignalFixtureFactory::new(move || {
            Ok(SignalHarnessRuntime {
                graph: graph.clone(),
                evaluator: Arc::clone(&evaluator),
                checked_evaluator: checked_evaluator.clone(),
                execution_authority: execution_authority.clone(),
                lease_request: lease_request.clone(),
                labels: labels.clone(),
            })
        });

        let mut plan = ScenarioPlan::new(name, fixture);
        for input in declared_inputs {
            plan = plan.declare_input(input);
        }
        for observation in declared_observations {
            plan = plan.declare_observation(observation);
        }
        for (key, value) in metadata {
            plan = plan.with_metadata(key, value);
        }
        Ok(plan.compile())
    }

    pub fn target_request(
        &self,
        name: impl Into<String>,
        target: impl Into<String>,
    ) -> ExecutionRequest<String> {
        ExecutionRequest::target(name, target.into())
    }

    pub fn request(
        &self,
        name: impl Into<String>,
        targets: impl IntoIterator<Item = impl Into<String>>,
    ) -> ExecutionRequest<String> {
        ExecutionRequest::new(name, targets.into_iter().map(Into::into).collect())
    }
}

impl SignalScenario {
    fn pending_dependencies_for(&self, node: NodeId) -> Result<Vec<DependencyEdge>, SignalError> {
        match self.pending_dependencies.get(&node) {
            Some(dependencies) => Ok(dependencies.clone()),
            None => Ok(self.graph.dependencies_of(node)?.to_vec()),
        }
    }

    fn flush_pending_dependencies(&mut self) -> Result<(), SignalError> {
        if self.pending_dependencies.is_empty() {
            return Ok(());
        }

        let pending = std::mem::take(&mut self.pending_dependencies)
            .into_iter()
            .collect::<Vec<_>>();
        self.graph
            .apply_dependency_batch_edit(&DependencyBatchEdit::from_pairs(pending))
    }
}

pub struct SignalMutationBatch {
    batch: MutationBatch<SignalMutationAction>,
}

impl SignalMutationBatch {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            batch: MutationBatch::new(name),
        }
    }

    pub fn action(mut self, action: SignalMutationAction) -> Self {
        self.batch = self.batch.push(action);
        self
    }

    pub fn mark_dirty(self, label: impl Into<String>, aspect: Aspect) -> Self {
        let label = label.into();
        self.action(SignalMutationAction::mark_dirty(
            format!("mark-{label}-dirty"),
            label,
            aspect,
        ))
    }

    pub fn mark_dirty_with_regions(
        self,
        label: impl Into<String>,
        aspect: Aspect,
        changed_regions: Vec<ChangedRegion>,
    ) -> Self {
        let label = label.into();
        self.action(SignalMutationAction::mark_dirty_with_regions(
            format!("mark-{label}-dirty-with-regions"),
            label,
            aspect,
            changed_regions,
        ))
    }

    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.batch = self.batch.with_metadata(key, value);
        self
    }

    pub fn build(self) -> MutationBatch<SignalMutationAction> {
        self.batch
    }
}
