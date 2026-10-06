//! Checked observations for bounded graph kernels. There is deliberately no
//! raw graph or whole-version accessor on this surface.

use worth_execution::MapKernelContext;

use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::BoundedSignalInputs;
use crate::data::output::PartitionSubscription;
use crate::data::retained_storage::{RetainedStoragePreparation, RetainedStoragePreparationDenial};
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::prepared::{PreparedDependencyCapture, PreparedEvaluation};

pub struct CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx> {
    graph: &'graph SignalGraph,
    node: NodeId,
    domain: &'graph Ctx,
    declaration: &'graph BoundedSignalInputs,
    dependencies: PreparedDependencyCapture,
    work: &'work mut MapKernelContext<'run, 'lease>,
    scratch_capacity_bytes: u64,
    result_capacity_bytes: u64,
    rejected: bool,
}

impl<'graph, 'work, 'run, 'lease, Ctx> CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx> {
    pub(crate) fn new(
        graph: &'graph SignalGraph,
        node: NodeId,
        domain: &'graph Ctx,
        work: &'work mut MapKernelContext<'run, 'lease>,
        scratch_capacity_bytes: u64,
        result_capacity_bytes: u64,
    ) -> Result<Self, SignalError> {
        let declaration = graph
            .get_contract(node)?
            .execution
            .bounded_inputs
            .as_ref()
            .ok_or_else(|| SignalError::invalid_input("checked evaluation needs bounded inputs"))?;
        work.checkpoint(declaration.copy_work_bound() as u64)
            .map_err(|_| SignalError::invalid_input("checked capture stopped before allocation"))?;
        Ok(Self {
            graph,
            node,
            domain,
            declaration,
            dependencies: PreparedDependencyCapture::with_capacity(declaration.as_slice().len()),
            work,
            scratch_capacity_bytes,
            result_capacity_bytes,
            rejected: false,
        })
    }

    pub fn node(&self) -> NodeId {
        self.node
    }
    pub fn domain(&self) -> &'graph Ctx {
        self.domain
    }
    pub fn work(&mut self) -> &mut MapKernelContext<'run, 'lease> {
        self.work
    }

    /// Maximum additional scratch the caller's checked kernel may allocate.
    /// Signal's own observation capture is reserved separately.
    pub fn scratch_capacity_bytes(&self) -> u64 {
        self.scratch_capacity_bytes
    }

    /// Maximum retained heap in the returned output, including trace metadata.
    /// Signal's captured dependency heap is reserved separately.
    pub fn result_capacity_bytes(&self) -> u64 {
        self.result_capacity_bytes
    }

    pub fn finish(
        &self,
        result: impl IntoEvaluationOutput,
    ) -> crate::logic::evaluation::EvaluationOutput {
        result.into_evaluation_output()
    }

    pub fn read(&mut self, source: NodeId, aspect: Aspect) -> Result<u64, SignalError> {
        self.observe(source, aspect, None)
    }

    pub fn read_scoped(
        &mut self,
        source: NodeId,
        aspect: Aspect,
        scope: &PartitionSubscription,
    ) -> Result<u64, SignalError> {
        self.observe(source, aspect, Some(scope))
    }

    fn observe(
        &mut self,
        source: NodeId,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
    ) -> Result<u64, SignalError> {
        if self.rejected {
            return Err(rejected_read());
        }
        // A conservative bound covers declaration search and capture insertion;
        // it is claimed before reading a version or growing the capture vector.
        if self
            .work
            .checkpoint(self.declaration.copy_work_bound() as u64 + 1)
            .is_err()
        {
            self.rejected = true;
            return Err(SignalError::invalid_input(
                "checked observation exhausted its work",
            ));
        }
        if !self.declaration.contains(source, aspect, scope) {
            self.rejected = true;
            return Err(rejected_read());
        }
        match self.graph.node_version_for_scope(source, aspect, scope) {
            Ok(value) => {
                self.dependencies.record(source, aspect, scope.cloned());
                Ok(value)
            }
            Err(error) => {
                self.rejected = true;
                Err(error)
            }
        }
    }

    pub(crate) fn into_prepared(
        self,
        output: impl IntoEvaluationOutput,
    ) -> Result<PreparedEvaluation, SignalError> {
        if self.rejected {
            return Err(rejected_read());
        }
        let prepared = output
            .into_evaluation_output()
            .into_prepared(self.dependencies);
        if let Some(maximum) = self
            .graph
            .get_contract(self.node)?
            .execution
            .max_checked_result_heap_bytes
        {
            let mut measurement = RetainedStoragePreparation::new(usize::MAX);
            let mut checkpoint = |visits: usize| {
                let units = u64::try_from(visits).map_err(|_| {
                    RetainedStoragePreparationDenial::WorkExhausted {
                        maximum_visits: usize::MAX,
                    }
                })?;
                self.work.checkpoint(units).map_err(|_| {
                    RetainedStoragePreparationDenial::WorkExhausted {
                        maximum_visits: usize::MAX,
                    }
                })
            };
            let mut observed = measurement.reborrow_with_checkpoint(&mut checkpoint);
            let actual = prepared
                .checked_result_heap_charge(&mut observed)
                .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            if actual.bytes() > maximum {
                return Err(SignalError::CheckedResultCapacityExceeded {
                    required: actual.bytes(),
                    declared: maximum,
                });
            }
        }
        Ok(prepared)
    }
}

fn rejected_read() -> SignalError {
    SignalError::invalid_input("checked evaluation observed an undeclared input")
}
