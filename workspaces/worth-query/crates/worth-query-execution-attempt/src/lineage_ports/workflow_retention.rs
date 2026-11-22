//! Workflow compilation-reuse and instance-progress retention store.

use std::any::Any;
use std::sync::{Arc, Mutex};

use worth_query_execution_lineage::facade::ports::WorkflowRetentionPort;

use crate::domain_computation::primary_graph::workflow::{
    definition::WorkflowDefinitionCompilationReuse,
    instance::progression::retention::{
        default_progress_retention_shards, WorkflowInstanceProgressKey,
        WorkflowInstanceProgressRetention,
    },
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphIntegrationHandle, WorthQueryWorkflowCompilationReuseCounters,
    WorthQueryWorkflowInstanceProgressCounters,
};

pub struct WorkflowRetentionStore {
    compilation_reuse: Mutex<WorkflowDefinitionCompilationReuse>,
    instance_progress: Arc<[Mutex<WorkflowInstanceProgressRetention>]>,
}

impl WorkflowRetentionStore {
    pub fn new() -> Self {
        Self {
            compilation_reuse: Mutex::new(WorkflowDefinitionCompilationReuse::default()),
            instance_progress: default_progress_retention_shards(),
        }
    }
}

impl WorkflowRetentionPort for WorkflowRetentionStore {
    fn compilation_reuse_counters(&self) -> WorthQueryWorkflowCompilationReuseCounters {
        self.compilation_reuse
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .counters()
    }

    fn instance_progress_counters(&self) -> WorthQueryWorkflowInstanceProgressCounters {
        let mut counters = WorthQueryWorkflowInstanceProgressCounters::default();
        for shard in self.instance_progress.iter() {
            let shard = shard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            counters.absorb(shard.counters());
        }
        counters
    }

    fn release_instance_progress(&self) {
        for shard in self.instance_progress.iter() {
            shard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .release_all();
        }
    }

    fn release_instance_progress_for_branch(&self, branch_occurrence: u64) {
        for shard in self.instance_progress.iter() {
            shard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .release_branch(branch_occurrence);
        }
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

pub trait PrimaryGraphHandleWorkflowExt {
    fn with_workflow_compilation_reuse_mut<T>(
        &self,
        mutate: impl FnOnce(&mut WorkflowDefinitionCompilationReuse) -> T,
    ) -> T;

    fn with_workflow_instance_progress_mut<T>(
        &self,
        key: WorkflowInstanceProgressKey,
        mutate: impl FnOnce(&mut WorkflowInstanceProgressRetention) -> T,
    ) -> T;
}

impl PrimaryGraphHandleWorkflowExt for WorthQueryPrimaryGraphIntegrationHandle {
    fn with_workflow_compilation_reuse_mut<T>(
        &self,
        mutate: impl FnOnce(&mut WorkflowDefinitionCompilationReuse) -> T,
    ) -> T {
        let store = self
            .workflow_retention()
            .as_any()
            .downcast_ref::<WorkflowRetentionStore>()
            .expect("primary graph carries WorkflowRetentionStore");
        let mut reuse = store
            .compilation_reuse
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mutate(&mut reuse)
    }

    fn with_workflow_instance_progress_mut<T>(
        &self,
        key: WorkflowInstanceProgressKey,
        mutate: impl FnOnce(&mut WorkflowInstanceProgressRetention) -> T,
    ) -> T {
        let store = self
            .workflow_retention()
            .as_any()
            .downcast_ref::<WorkflowRetentionStore>()
            .expect("primary graph carries WorkflowRetentionStore");
        let shard = &store.instance_progress[key.shard_index()];
        let mut retention = shard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mutate(&mut retention)
    }
}
