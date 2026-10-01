use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::{ExecutionAuthority, LeaseRequest};

use crate::facade::*;

pub trait SignalEvaluationDriver: Send + Sync {
    fn evaluate(
        &self,
        ctx: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError>;
}

pub trait SignalCheckedEvaluationDriver: Send + Sync {
    fn evaluate_checked(
        &self,
        ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>,
    ) -> Result<EvaluationOutput, SignalError>;
}

impl<F, O> SignalCheckedEvaluationDriver for F
where
    F: for<'graph, 'work, 'run, 'lease> Fn(
            &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, ()>,
        ) -> Result<O, SignalError>
        + Send
        + Sync,
    O: IntoEvaluationOutput,
{
    fn evaluate_checked(
        &self,
        ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        self(ctx).map(IntoEvaluationOutput::into_evaluation_output)
    }
}

impl<F, O> SignalEvaluationDriver for F
where
    F: for<'a> Fn(&mut EvaluationContext<'a, ()>) -> Result<O, SignalError> + Send + Sync,
    O: IntoEvaluationOutput,
{
    fn evaluate(
        &self,
        ctx: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        self(ctx).map(IntoEvaluationOutput::into_evaluation_output)
    }
}

#[derive(Clone)]
pub struct SignalFixtureFactory {
    builder: Arc<dyn Fn() -> Result<SignalHarnessRuntime, SignalError> + Send + Sync>,
}

impl SignalFixtureFactory {
    pub fn new<F>(builder: F) -> Self
    where
        F: Fn() -> Result<SignalHarnessRuntime, SignalError> + Send + Sync + 'static,
    {
        Self {
            builder: Arc::new(builder),
        }
    }

    pub fn build_runtime(&self) -> Result<SignalHarnessRuntime, SignalError> {
        (self.builder)()
    }
}

#[derive(Clone)]
pub struct SignalMutationAction {
    name: String,
    kind: SignalMutationKind,
}

#[derive(Clone)]
pub(crate) enum SignalMutationKind {
    MarkDirty {
        label: String,
        aspect: Aspect,
    },
    MarkDirtyWithRegions {
        label: String,
        aspect: Aspect,
        changed_regions: Vec<ChangedRegion>,
    },
}

impl SignalMutationAction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn kind(&self) -> &SignalMutationKind {
        &self.kind
    }

    pub fn mark_dirty(name: impl Into<String>, label: impl Into<String>, aspect: Aspect) -> Self {
        Self {
            name: name.into(),
            kind: SignalMutationKind::MarkDirty {
                label: label.into(),
                aspect,
            },
        }
    }

    pub fn mark_dirty_with_regions(
        name: impl Into<String>,
        label: impl Into<String>,
        aspect: Aspect,
        changed_regions: Vec<ChangedRegion>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: SignalMutationKind::MarkDirtyWithRegions {
                label: label.into(),
                aspect,
                changed_regions,
            },
        }
    }
}

pub struct SignalHarnessRuntime {
    pub(crate) graph: SignalGraph,
    pub(crate) evaluator: Arc<dyn SignalEvaluationDriver>,
    pub(crate) checked_evaluator: Option<Arc<dyn SignalCheckedEvaluationDriver>>,
    pub(crate) execution_authority: Option<Arc<ExecutionAuthority>>,
    pub(crate) lease_request: Option<LeaseRequest>,
    pub(crate) labels: BTreeMap<String, NodeId>,
}

impl SignalHarnessRuntime {
    pub fn builder() -> SignalHarnessRuntimeBuilder {
        SignalHarnessRuntimeBuilder::new()
    }

    pub fn graph(&self) -> &SignalGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut SignalGraph {
        &mut self.graph
    }

    pub fn label(&mut self, name: impl Into<String>, node: NodeId) {
        self.labels.insert(name.into(), node);
    }

    pub fn resolve(&self, label: &str) -> Result<NodeId, SignalError> {
        self.labels.get(label).copied().ok_or_else(|| {
            SignalError::invalid_input(format!("unknown harness target label `{label}`"))
        })
    }

    pub fn labels(&self) -> &BTreeMap<String, NodeId> {
        &self.labels
    }
}

pub struct SignalHarnessRuntimeBuilder {
    graph: SignalGraph,
    evaluator: Option<Arc<dyn SignalEvaluationDriver>>,
    checked_evaluator: Option<Arc<dyn SignalCheckedEvaluationDriver>>,
    execution_authority: Option<Arc<ExecutionAuthority>>,
    lease_request: Option<LeaseRequest>,
    labels: BTreeMap<String, NodeId>,
}

impl Default for SignalHarnessRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalHarnessRuntimeBuilder {
    pub fn new() -> Self {
        Self {
            graph: SignalGraph::new(),
            evaluator: None,
            checked_evaluator: None,
            execution_authority: None,
            lease_request: None,
            labels: BTreeMap::new(),
        }
    }

    pub fn graph(&self) -> &SignalGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut SignalGraph {
        &mut self.graph
    }

    pub fn label(mut self, name: impl Into<String>, node: NodeId) -> Self {
        self.labels.insert(name.into(), node);
        self
    }

    pub fn insert_label(&mut self, name: impl Into<String>, node: NodeId) {
        self.labels.insert(name.into(), node);
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

    pub fn build(self) -> Result<SignalHarnessRuntime, SignalError> {
        let evaluator = self.evaluator.ok_or_else(|| {
            SignalError::invalid_input("signal harness runtime requires an evaluator")
        })?;
        Ok(SignalHarnessRuntime {
            graph: self.graph,
            evaluator,
            checked_evaluator: self.checked_evaluator,
            execution_authority: self.execution_authority,
            lease_request: self.lease_request,
            labels: self.labels,
        })
    }
}

pub struct SignalHarnessSession {
    pub(crate) runtime: Option<SignalHarnessRuntime>,
}

impl SignalHarnessSession {
    pub(crate) fn runtime(&self) -> Result<&SignalHarnessRuntime, SignalError> {
        self.runtime.as_ref().ok_or_else(|| {
            SignalError::invalid_input("signal harness fixture must be loaded before use")
        })
    }

    pub(crate) fn runtime_mut(&mut self) -> Result<&mut SignalHarnessRuntime, SignalError> {
        self.runtime.as_mut().ok_or_else(|| {
            SignalError::invalid_input("signal harness fixture must be loaded before use")
        })
    }
}
