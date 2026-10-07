use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::domain_installation::execution_index::WorthQueryWorkflowExecutionDescriptor;
use crate::runtime::WorthQueryWorkspace;

use super::{
    WorthQueryDomainWorkflowStageExecutor, WorthQueryExecutableDomainOperation,
    WorthQueryWorkflowOperation, WorthQueryWorkflowStageComputationFailure,
    WorthQueryWorkflowStageComputed, WorthQueryWorkflowStageExecutionContext,
    WorthQueryWorkflowStageExecutorFailure, WorthQueryWorkflowStageMaterial,
    WorthQueryWorkflowStagePreparation, WorthQueryWorkflowStageTask,
};

#[path = "stage_executor_registry/phase_dispatch.rs"]
mod phase_dispatch;
use super::workflow_parallel_progression::frontier::CanonicalWorkflowStageResult;
use phase_dispatch::TypedWorkflowStageExecutor;

type StagePreparation =
    for<'a> fn(
        WorthQueryWorkflowStagePreparation<'a>,
    )
        -> Result<WorthQueryWorkflowStageTask, WorthQueryWorkflowStageComputationFailure>;
type StageComputation = fn(
    WorthQueryWorkflowStageTask,
    &mut worth_execution::MapKernelContext<'_, '_>,
) -> WorthQueryWorkflowStageComputed;

trait ErasedWorkflowStageExecutor: Send + Sync {
    fn idempotent_stage_retry(&self) -> bool;
    fn apply(
        &self,
        slot: CanonicalWorkflowStageResult,
        context: &WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowStageMaterial, WorthQueryWorkflowStageExecutorFailure>;
}

pub(crate) trait ErasedReplaySemanticComparator: Send + Sync {
    fn compare(
        &self,
        original: &super::WorthQueryWorkflowTraceSemantics,
        replay: &super::WorthQueryWorkflowTraceSemantics,
        noise: super::WorthQueryReplayNoiseContract,
    ) -> super::WorthQueryReplayComparison;
}

pub(crate) struct WorthQueryInstalledWorkflowStageExecutor {
    executor: Arc<dyn ErasedWorkflowStageExecutor>,
    preparation: StagePreparation,
    computation: StageComputation,
    replay_comparator: Option<Arc<dyn ErasedReplaySemanticComparator>>,
    pub(crate) installed_read: Option<crate::ordinary::read::WorthQueryReadDeclaration>,
    pub(crate) resource_support: super::WorthQueryExecutionResourceSupport,
}

struct WorkflowStageExecutorRegistration {
    executor: Arc<dyn ErasedWorkflowStageExecutor>,
    preparation: StagePreparation,
    computation: StageComputation,
    replay_comparator: Option<Arc<dyn ErasedReplaySemanticComparator>>,
    lowering_family: &'static str,
    deterministic: bool,
    execution_cost: crate::domain_installation::WorthQueryOperationCostClass,
    result_width_cost: crate::domain_installation::WorthQueryOperationCostClass,
    replay_comparator_family: Option<&'static str>,
    installed_read: Option<crate::ordinary::read::WorthQueryReadDeclaration>,
    resource_support: super::WorthQueryExecutionResourceSupport,
}

impl WorthQueryInstalledWorkflowStageExecutor {
    pub(crate) fn prepare(
        &self,
        view: WorthQueryWorkflowStagePreparation<'_>,
    ) -> Result<WorthQueryWorkflowStageTask, WorthQueryWorkflowStageComputationFailure> {
        (self.preparation)(view)
    }

    pub(crate) fn computation(
        &self,
    ) -> fn(
        WorthQueryWorkflowStageTask,
        &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> WorthQueryWorkflowStageComputed {
        self.computation
    }

    pub(crate) fn idempotent_stage_retry(&self) -> bool {
        self.executor.idempotent_stage_retry()
    }

    pub(in crate::domain_installation::operation_execution) fn apply(
        &self,
        slot: CanonicalWorkflowStageResult,
        context: &WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowStageMaterial, WorthQueryWorkflowStageExecutorFailure> {
        self.executor.apply(slot, context, workspace)
    }

    pub(crate) fn replay_comparator(&self) -> Option<Arc<dyn ErasedReplaySemanticComparator>> {
        self.replay_comparator.as_ref().map(Arc::clone)
    }
}

#[derive(Default)]
pub(crate) struct WorthQueryPendingWorkflowStageExecutors {
    registrations: HashMap<(TypeId, TypeId, TypeId), WorkflowStageExecutorRegistration>,
    duplicate: bool,
}

impl WorthQueryPendingWorkflowStageExecutors {
    pub(crate) fn register<
        D: 'static,
        O,
        F: 'static,
        E: WorthQueryDomainWorkflowStageExecutor<D, O, F>,
    >(
        mut self,
        executor: E,
    ) -> Self
    where
        O: 'static
            + WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
    {
        let installed_read = executor
            .installed_read_declaration()
            .map(|declaration| declaration.clone_for_installed_execution());
        let resource_support = executor.execution_resource_support();
        let typed = Arc::new(TypedWorkflowStageExecutor::<D, O, F, E> {
            executor: Arc::new(executor),
            marker: PhantomData,
        });
        self.insert_registration::<D, O, F>(WorkflowStageExecutorRegistration {
            installed_read,
            preparation: E::prepare,
            computation: E::compute,
            executor: typed,
            replay_comparator: None,
            lowering_family: E::LOWERING_FAMILY,
            deterministic: E::DETERMINISTIC,
            execution_cost: E::EXECUTION_COST,
            result_width_cost: E::RESULT_WIDTH_COST,
            replay_comparator_family: None,
            resource_support,
        });
        self
    }

    pub(crate) fn register_replayable<
        D: 'static,
        O,
        F: 'static,
        E: WorthQueryDomainWorkflowStageExecutor<D, O, F>
            + super::WorthQueryDomainReplaySemanticComparator<D, O, F>,
    >(
        mut self,
        executor: E,
    ) -> Self
    where
        O: 'static
            + WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
    {
        let installed_read = executor
            .installed_read_declaration()
            .map(|declaration| declaration.clone_for_installed_execution());
        let resource_support = executor.execution_resource_support();
        let typed = Arc::new(TypedWorkflowStageExecutor::<D, O, F, E> {
            executor: Arc::new(executor),
            marker: PhantomData,
        });
        self.insert_registration::<D, O, F>(WorkflowStageExecutorRegistration {
            installed_read,
            preparation: E::prepare,
            computation: E::compute,
            executor: typed.clone(),
            replay_comparator: Some(typed),
            lowering_family: E::LOWERING_FAMILY,
            deterministic: E::DETERMINISTIC,
            execution_cost: E::EXECUTION_COST,
            result_width_cost: E::RESULT_WIDTH_COST,
            replay_comparator_family: E::REPLAY_COMPARATOR_FAMILY,
            resource_support,
        });
        self
    }

    fn insert_registration<D: 'static, O: 'static, F: 'static>(
        &mut self,
        registration: WorkflowStageExecutorRegistration,
    ) {
        let key = (TypeId::of::<D>(), TypeId::of::<O>(), TypeId::of::<F>());
        self.duplicate |= self.registrations.insert(key, registration).is_some();
    }

    pub(crate) fn install(
        self,
        workflow_operations: &[WorthQueryWorkflowExecutionDescriptor],
    ) -> Result<WorthQueryWorkflowStageExecutorRegistry, &'static str> {
        if self.duplicate {
            return Err("duplicate exact workflow stage executor registration");
        }
        let expected = workflow_operations
            .iter()
            .map(|descriptor| (descriptor.domain, descriptor.operation, descriptor.family))
            .collect::<HashSet<_>>();
        let actual = self.registrations.keys().copied().collect::<HashSet<_>>();
        if expected != actual {
            return Err("installed workflow operation and stage executor registration sets differ");
        }
        for descriptor in workflow_operations {
            if descriptor.has_unsupported_effect_family {
                return Err(
                    "workflow declares an effect family without a Query-owned stage execution door",
                );
            }
            let registration = self
                .registrations
                .get(&(descriptor.domain, descriptor.operation, descriptor.family))
                .expect("workflow executor registration set was closed");
            if registration.lowering_family != descriptor.lowering_family {
                return Err(
                    "workflow stage executor lowering family disagrees with installed semantics",
                );
            }
            if registration.deterministic != descriptor.deterministic_lowering {
                return Err("workflow executor determinism disagrees with installed semantics");
            }
            if descriptor.lookup_cost
                != crate::domain_installation::WorthQueryOperationCostClass::Constant
            {
                return Err("installed workflow lookup cost is not constant-time indexed lookup");
            }
            if registration.execution_cost != descriptor.execution_cost
                || registration.result_width_cost != descriptor.result_width_cost
            {
                return Err("workflow executor cost contract disagrees with installed semantics");
            }
            if registration.replay_comparator_family
                != descriptor.replay_comparator_family.as_deref()
            {
                return Err("workflow replay comparator disagrees with installed semantics");
            }
            match (&registration.installed_read, descriptor.requires_installed_read) {
                (Some(declaration), true)
                    if declaration.identity().canonical_query_digest() == descriptor.query_digest
                        && declaration.identity().canonical_result_shape_digest()
                            == descriptor.result_shape_digest => {}
                (Some(_), true) => return Err(
                    "workflow executor read declaration disagrees with installed canonical semantics",
                ),
                (None, true) => {
                    return Err("workflow executor is missing its installed read declaration")
                }
                (Some(_), false) => {
                    return Err("workflow executor registered an undeclared installed read plan")
                }
                (None, false) => {}
            }
        }
        Ok(WorthQueryWorkflowStageExecutorRegistry {
            registrations: self
                .registrations
                .into_iter()
                .map(|(key, registration)| {
                    (
                        key,
                        Arc::new(WorthQueryInstalledWorkflowStageExecutor {
                            executor: registration.executor,
                            preparation: registration.preparation,
                            computation: registration.computation,
                            replay_comparator: registration.replay_comparator,
                            installed_read: registration.installed_read,
                            resource_support: registration.resource_support,
                        }),
                    )
                })
                .collect(),
        })
    }
}

pub(crate) struct WorthQueryWorkflowStageExecutorRegistry {
    registrations: HashMap<(TypeId, TypeId, TypeId), Arc<WorthQueryInstalledWorkflowStageExecutor>>,
}

impl WorthQueryWorkflowStageExecutorRegistry {
    pub(crate) fn get<D: 'static, O: 'static, F: 'static>(
        &self,
    ) -> Option<Arc<WorthQueryInstalledWorkflowStageExecutor>> {
        self.registrations
            .get(&(TypeId::of::<D>(), TypeId::of::<O>(), TypeId::of::<F>()))
            .map(Arc::clone)
    }
}
