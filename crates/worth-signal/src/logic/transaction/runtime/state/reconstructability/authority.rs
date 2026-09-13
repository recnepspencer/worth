use super::super::resource::ResourceRuntimeState;
use super::super::temporal::TemporalRuntimeState;
use crate::data::graph::SignalGraph;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::transaction::runtime::config::SignalRuntimeConfig;

#[derive(Debug, Clone)]
pub(in crate::logic::transaction::runtime) struct AuthorityState<T>
where
    T: Copy + Ord,
{
    pub graph: SignalGraph,
    pub config: SignalRuntimeConfig<T>,
}

impl<T> AuthorityState<T>
where
    T: Copy + Ord,
{
    pub fn capture(graph: &SignalGraph, config: &SignalRuntimeConfig<T>) -> Self {
        Self {
            graph: graph.clone_stateful(),
            config: config.clone(),
        }
    }

    pub(in crate::logic::transaction::runtime) fn into_parts(
        self,
    ) -> (SignalGraph, SignalRuntimeConfig<T>) {
        (self.graph, self.config)
    }
}

#[derive(Debug, Clone)]
pub(in crate::logic::transaction::runtime) struct DerivedState<D, I>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
{
    pub checkpoint: CheckpointRuntime<D, I>,
    pub resource: ResourceRuntimeState,
    pub temporal: TemporalRuntimeState,
    pub telemetry: RuntimeTelemetry,
}

impl<D, I> DerivedState<D, I>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
{
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            checkpoint: self.checkpoint.fork_persistent(),
            resource: self.resource.fork_persistent(),
            temporal: self.temporal.fork_persistent(),
            telemetry: self.telemetry,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            checkpoint: self.checkpoint.fork_storage_identity(),
            resource: self.resource.fork_storage_identity(),
            temporal: self.temporal.fork_storage_identity(),
            telemetry: self.telemetry,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_fork_storage_with(&self, other: &Self) -> bool {
        self.checkpoint.shares_storage_with(&other.checkpoint)
            && self.resource.shares_storage_with(&other.resource)
            && self.temporal.shares_storage_with(&other.temporal)
    }

    pub fn capture(
        checkpoint: &CheckpointRuntime<D, I>,
        resource: &ResourceRuntimeState,
        temporal: &TemporalRuntimeState,
        telemetry: &RuntimeTelemetry,
    ) -> Self {
        Self {
            checkpoint: checkpoint.clone(),
            resource: resource.clone(),
            temporal: temporal.clone(),
            telemetry: *telemetry,
        }
    }
}
