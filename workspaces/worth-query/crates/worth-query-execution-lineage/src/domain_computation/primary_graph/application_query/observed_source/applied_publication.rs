//! Candidate source publication meaning retained across preparation.

use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

use super::source_identity::WorthQueryObservedSourceMeaning;
use super::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture;
use crate::ports::{AppliedManagedSourcePort, SourceReadClosurePort};

#[derive(Clone)]
pub struct AppliedSourcePublication {
    pub managed: Option<Arc<dyn AppliedManagedSourcePort>>,
    pub meaning: Arc<WorthQueryObservedSourceMeaning>,
    pub closure: Arc<dyn SourceReadClosurePort>,
    pub lifecycle: WorthQueryProducerLifecyclePosture,
}

impl AppliedSourcePublication {
    pub fn lifecycle(&self) -> WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }

    pub fn meaning(&self) -> &Arc<WorthQueryObservedSourceMeaning> {
        &self.meaning
    }

    pub fn closure(&self) -> &Arc<dyn SourceReadClosurePort> {
        &self.closure
    }

    pub fn root(&self) -> worth_relational::facade::identity::EntityId {
        self.meaning.footprint().root
    }

    pub fn checkpoint_identity(&self) -> [u8; 32] {
        self.meaning.checkpoint_identity().bytes()
    }

    pub fn seal_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial> {
        if let Some(managed) = &self.managed {
            managed.seal_published(execution, runtime, after)?;
        }
        self.closure.seal_applied_publication(runtime, after)?;
        Ok(())
    }

    pub fn matches_published_source<Query>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryObservedSource<Query>,
    ) -> Result<bool, Cause> {
        if !self.meaning.same_coordinate(&source.source_meaning) {
            return Ok(false);
        }
        match (&self.managed, source.invocation_managed_source.as_deref()) {
            (Some(managed), Some(current)) => managed.matches_current(execution, current),
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }
}
