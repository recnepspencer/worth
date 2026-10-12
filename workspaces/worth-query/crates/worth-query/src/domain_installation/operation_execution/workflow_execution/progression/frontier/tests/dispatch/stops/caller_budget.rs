//! Caller work bounds the complete frontier, including native application.
use super::*;

fn first_stage_owner_work(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    probe: &phased_workflow::fold_executor::ComputeProbe,
) -> u64 {
    // An independent native application control quotes the same insert and
    // installed read. No frontier, per-task ceiling or worker width supplies it.
    let mut workspace = Policy::width(0).workspace(request_execution, probe);
    let authority = probe.authority();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, 1024),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let ((), report) = execution
        .run(worth_execution::ExecutionWorkCeiling::new(1024), |_| {
            let command = crate::runtime::WorthQueryAspectMutationBuilder::new()
                .aspect("identity.id", "left:50")
                .build_insert("Vertex")
                .unwrap();
            workspace
                .execute_ordinary_authoritative_mutation(&execution, command, false)
                .unwrap();
            phased_workflow::world::installed_read_declaration()
                .clone_for_installed_execution()
                .using(crate::ordinary::read::current())
                .run(&execution, &mut workspace)
                .into_result()
                .unwrap();
        })
        .unwrap();
    report.charged_work()
}

#[test]
fn caller_work_ceiling_keeps_the_same_typed_boundary_at_all_widths() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let owner_work = first_stage_owner_work(&execution, &probe);
    for workers in [1, 2, 4] {
        let mut workspace = Policy {
            work: COSTS[0] + owner_work,
            ..Policy::width(workers)
        }
        .workspace(&execution, &probe);
        let mut input = members(7, None);
        // The second task requires more than the room reserved for application.
        // Its declaration equals its work; only the caller ceiling is short.
        input[1].1 = domain::WorthQueryWorkflowValue::Text(format!("8:0:{}", owner_work + 1));
        let (record, work, failed) =
            observe_members(start(&mut workspace), input, &mut workspace, None);
        assert_eq!(
            (denial(&record), failed),
            (
                &Denial::ComputationWorkExhausted {
                    stage_identity: Some("middle".into()),
                    cause: MapKernelStop::WorkCeiling,
                },
                false
            ),
            "width {workers}: {record:?}"
        );
        assert_eq!(work, COSTS[0]);
        assert_eq!(record.receipts.len(), 2);
        assert_eq!(record.effects.len(), 1);
    }
}

#[test]
fn unfunded_native_effect_is_a_typed_caller_work_stop_at_all_widths() {
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for workers in [1, 2, 4] {
        let mut workspace = Policy {
            work: COSTS[0],
            ..Policy::width(workers)
        }
        .workspace(&execution, &probe);
        let (record, work, failed) = observe_members(
            start(&mut workspace),
            members(7, None),
            &mut workspace,
            None,
        );
        let expected = Denial::StageExecutionRequest {
            stage_identity: "left".into(),
            cause: Ok(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(u64::from(worth_relational::facade::identity::PartitionId::main().0)),
                policy_ancestor: None,
            }),
        };
        assert_eq!(
            (denial(&record), failed),
            (&expected, false),
            "width {workers}: {record:?}"
        );
        assert_eq!(work, COSTS[0]);
        assert_eq!(record.receipts.len(), 1);
        assert_eq!(record.effects.len(), 0);
    }
}
