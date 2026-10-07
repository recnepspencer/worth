use super::{reference_executor, world, PhaseDomain, PhaseFamily, PhaseOperation};
use worth_query::facade::{domain, read, runtime};
#[path = "compute_probe.rs"]
mod compute_probe;
pub(crate) use compute_probe::ComputeProbe;

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
        let count = input
            .split(':')
            .nth(2)
            .map_or(17, |count| count.parse::<u64>().unwrap());
        if mode == 1 {
            return Err(computation_failure("prepare failure"));
        }
        let words = std::iter::once(mode)
            .chain((0..count).map(|offset| seed.wrapping_add(offset)))
            .collect();
        Ok(view
            .task(domain::WorthQueryWorkflowStageComputePayload::Words(words))
            .with_computation_limits(
                if mode == 9 { 0 } else { count },
                0,
                if mode == 11 {
                    128
                } else if mode == 7 || mode == 9 {
                    0
                } else {
                    (count + 1) * 8
                },
            ))
    }

    fn compute(
        task: domain::WorthQueryWorkflowStageTask,
        meter: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> domain::WorthQueryWorkflowStageComputed {
        let domain::WorthQueryWorkflowStageComputePayload::Words(words) = task.payload() else {
            return task.pass_through();
        };
        let mode = words[0];
        // This test-only probe never feeds a result: its counters and rendezvous
        // observe and schedule computation without changing its pure value.
        if !compute_probe::before(task.stage_identity()) {
            return task.pass_through();
        }
        if mode == 9 {
            compute_probe::completed(task.stage_identity(), 0);
            return task.complete(Ok(domain::WorthQueryWorkflowStageComputePayload::Empty));
        }
        let mut work = 0;
        let folded = words[1..]
            .iter()
            .enumerate()
            .try_fold(0_u64, |sum, (index, word)| {
                meter.checkpoint(1)?;
                work += 1;
                if mode == 8 {
                    compute_probe::cancel();
                    meter.checkpoint(0)?;
                }
                Ok::<_, worth_execution::MapKernelStop>(
                    sum.wrapping_add(word.wrapping_mul(index as u64 + 1)),
                )
            });
        compute_probe::completed(task.stage_identity(), work);
        let folded = match folded {
            Ok(folded) => folded,
            Err(stop) => return task.complete(Err(computation_failure(&format!("{stop:?}")))),
        };
        if mode == 6 {
            panic!("contained fixture compute panic");
        }
        if mode == 10 {
            let partitions = std::collections::BTreeMap::from([(
                worth_foundational::PartitionIdentity::new(0),
                worth_execution::KeylessPartition {
                    value: 0_u64,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )]);
            let nested =
                worth_execution::ExecutionMap::<_, ()>::from_keyless_partitions(partitions)
                    .unwrap()
                    .run_owned(None, |value, _| {
                        let _ = value;
                        Err::<u64, _>(worth_execution::MapKernelFailure::Domain(()))
                    });
            assert!(matches!(
                nested,
                worth_execution::MapOutcome::Stopped { .. }
            ));
            // Serial nested domain refusal and leased admission refusal both
            // stop the invoking meter; neither posture may panic at this door.
        }
        if mode == 11 {
            return task.complete(Err(domain::WorthQueryWorkflowStageComputationFailure::new(
                domain::WorthQueryOperationFailureClass::Domain("x".repeat(4096)),
                "domain failure",
            )));
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
