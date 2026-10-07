use super::{reference_executor, world, PhaseDomain, PhaseFamily, PhaseOperation};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use worth_query::facade::{domain, read, runtime};

type ComputeEvents = Mutex<Vec<(String, usize)>>;
static TEST_SERIALIZATION: Mutex<()> = Mutex::new(());
static COMPUTES: Mutex<Option<Weak<ComputeEvents>>> = Mutex::new(None);

pub(crate) struct ComputeProbe {
    _serialization: MutexGuard<'static, ()>,
    events: Arc<ComputeEvents>,
}
impl ComputeProbe {
    pub(crate) fn new() -> Self {
        let serialization = TEST_SERIALIZATION.lock().unwrap_or_else(|e| e.into_inner());
        let events = Arc::new(Mutex::new(Vec::new()));
        *COMPUTES.lock().unwrap() = Some(Arc::downgrade(&events));
        Self {
            _serialization: serialization,
            events,
        }
    }
}
impl Drop for ComputeProbe {
    fn drop(&mut self) {
        *COMPUTES.lock().unwrap() = None;
        assert_eq!(Arc::strong_count(&self.events), 1, "compute has finished");
    }
}
pub(crate) fn take_computes() -> Vec<(String, usize)> {
    let events = COMPUTES.lock().unwrap().as_ref().and_then(Weak::upgrade);
    events.map_or_else(Vec::new, |events| {
        std::mem::take(&mut *events.lock().unwrap())
    })
}

pub(crate) struct FoldExecutor;

impl domain::WorthQueryDomainWorkflowStageExecutor<PhaseDomain, PhaseOperation, PhaseFamily>
    for FoldExecutor
{
    const LOWERING_FAMILY: &'static str = "workflow-fold-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        world::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(world::installed_read_declaration())
    }

    fn prepare(
        view: domain::WorthQueryWorkflowStagePreparation<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageTask,
        domain::WorthQueryWorkflowStageComputationFailure,
    > {
        if !super::MEMBERS.contains(&view.stage_identity()) {
            return Ok(view.task(domain::WorthQueryWorkflowStageComputePayload::Empty));
        }
        let domain::WorthQueryWorkflowStageInputFacts::Text(input) = view.input() else {
            panic!("member input is text")
        };
        let (seed, mode) = reference_executor::parse(input);
        if mode == 1 {
            return Err(computation_failure("prepare failure"));
        }
        let words = std::iter::once(mode)
            .chain((0..17).map(|offset| seed.wrapping_add(offset)))
            .collect();
        Ok(view.task(domain::WorthQueryWorkflowStageComputePayload::Words(words)))
    }

    fn compute(
        task: domain::WorthQueryWorkflowStageTask,
    ) -> domain::WorthQueryWorkflowStageComputed {
        let domain::WorthQueryWorkflowStageComputePayload::Words(words) = task.payload() else {
            return task.pass_through();
        };
        let mode = words[0];
        let mut work = 0;
        let folded = words[1..]
            .iter()
            .enumerate()
            .fold(0_u64, |sum, (index, word)| {
                work += 1;
                sum.wrapping_add(word.wrapping_mul(index as u64 + 1))
            });
        // This test-only counter never feeds a result: observing computes adds no
        // owner effect or order-dependent behavior to the stage's computation.
        if let Some(events) = COMPUTES.lock().unwrap().as_ref().and_then(Weak::upgrade) {
            events
                .lock()
                .unwrap()
                .push((task.stage_identity().into(), work));
        }
        if mode == 2 {
            return task.complete(Err(computation_failure("compute failure")));
        }
        task.complete(Ok(domain::WorthQueryWorkflowStageComputePayload::Words(
            vec![mode, folded, work as u64],
        )))
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (input, computed, context, workspace) = application.into_parts();
        let domain::WorthQueryWorkflowStageComputePayload::Words(words) = computed else {
            return reference_executor::reference_step(input, context, workspace);
        };
        let stage = context.stage().identity();
        let (mode, folded) = (words[0], words[1]);
        if mode == 3 {
            return Err(reference_executor::failure("apply failure"));
        }
        let command = runtime::WorthQueryAspectMutationBuilder::new()
            .aspect("identity.id", format!("{stage}:{folded}"))
            .build_insert("Vertex")
            .unwrap();
        context
            .execute_mutation(command, workspace)
            .map_err(|denial| reference_executor::failure(format!("{denial:?}")))?;
        if mode == 4 {
            return Err(reference_executor::failure("failure after effect"));
        }
        let read = context.execute_installed_read("model", workspace)?;
        let output = if mode == 5 {
            domain::WorthQueryWorkflowValue::Bool(false)
        } else {
            domain::WorthQueryWorkflowValue::Text(format!(
                "{stage}:{folded}:{}",
                read.result().rows().len()
            ))
        };
        Ok(domain::WorthQueryWorkflowStageMaterial::new(output)
            .with_primary_graph_read("model", &read))
    }
}

fn computation_failure(detail: &str) -> domain::WorthQueryWorkflowStageComputationFailure {
    domain::WorthQueryWorkflowStageComputationFailure::new(
        domain::WorthQueryOperationFailureClass::Dependency,
        detail,
    )
}
