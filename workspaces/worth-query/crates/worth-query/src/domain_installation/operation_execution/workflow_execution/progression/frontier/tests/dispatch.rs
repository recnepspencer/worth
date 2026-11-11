//! Actual authority placement oracle, with no schedule hook in Query.
use super::differential::{record_billed as record, Record};
use super::{domain, phased_workflow, start};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_proof::TransitionOutcome;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
#[path = "dispatch/interruption.rs"]
mod interruption;
#[path = "dispatch/reservation.rs"]
mod reservation;
#[path = "dispatch/stops.rs"]
mod stops;
const COSTS: [u64; 3] = [3, 17, 41];

#[derive(Clone, Copy)]
struct Policy {
    workers: usize,
    memory: Option<u64>,
    work: Option<u64>,
}
impl Policy {
    fn width(workers: usize) -> Self {
        Self {
            workers,
            memory: None,
            work: None,
        }
    }
    fn workspace(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        probe: &phased_workflow::fold_executor::ComputeProbe,
    ) -> worth_query::facade::runtime::WorthQueryWorkspace {
        let work = self
            .work
            .unwrap_or_else(|| ordinary_owner_work_quote(request_execution, probe));
        self.workspace_with_work(request_execution, probe, Some(work))
    }
    fn workspace_with_work(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        probe: &phased_workflow::fold_executor::ComputeProbe,
        work: Option<u64>,
    ) -> worth_query::facade::runtime::WorthQueryWorkspace {
        let policy = ExecutionRequestPolicy::new(
            if self.workers == 0 {
                ExecutionPosture::Serial
            } else {
                ExecutionPosture::Automatic
            },
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                NonZeroUsize::new(self.workers).unwrap_or(NonZeroUsize::MIN),
                self.memory,
                work,
            ),
        );
        let resources = crate::consumer_kit::in_memory_test_product_world_resources()
            .with_execution_policy(policy);
        let resources = if self.workers == 0 {
            resources
        } else {
            resources.with_execution_authority(probe.authority())
        };
        phased_workflow::phased_workspace_with_resources(
            request_execution,
            "width-oracle",
            resources,
        )
    }
}
// Quote the actual Native applications and installed reads independently of
// frontier lowering and dispatch. Seed29 is the widest successful fixture input.
fn ordinary_owner_work_quote(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    probe: &phased_workflow::fold_executor::ComputeProbe,
) -> u64 {
    let mut workspace = Policy::width(0).workspace_with_work(request, probe, None);
    let authority = probe.authority();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, None, None),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let ((), report) = execution
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            for (index, stage) in phased_workflow::MEMBERS.iter().enumerate() {
                let seed = 29 + index as u64;
                let folded = (0..COSTS[index]).fold(0_u64, |sum, offset| {
                    sum.wrapping_add(seed.wrapping_add(offset).wrapping_mul(offset + 1))
                });
                let command = crate::runtime::WorthQueryAspectMutationBuilder::new()
                    .aspect("identity.id", format!("{stage}:{folded}"))
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
            }
        })
        .unwrap();
    report
        .charged_work()
        .checked_add(COSTS.iter().sum())
        .unwrap()
}
fn members(
    seed: u64,
    failure: Option<(usize, u64)>,
) -> Vec<(String, domain::WorthQueryWorkflowValue)> {
    phased_workflow::MEMBERS
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let mode = failure
                .filter(|(at, _)| *at == index)
                .map_or(0, |(_, mode)| mode);
            (
                name.to_string(),
                domain::WorthQueryWorkflowValue::Text(format!(
                    "{}:{mode}:{}",
                    seed + index as u64,
                    COSTS[index]
                )),
            )
        })
        .collect()
}
fn observe(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: u64,
    failure: Option<(usize, u64)>,
    policy: Policy,
    probe: &phased_workflow::fold_executor::ComputeProbe,
) -> (Record, u64, bool) {
    let mut workspace = policy.workspace(request_execution, probe);
    observe_members(
        start(&mut workspace),
        members(seed, failure),
        &mut workspace,
        None,
    )
}
fn observe_members(
    mut run: super::Run,
    members: Vec<(String, domain::WorthQueryWorkflowValue)>,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    scope: Option<&WorthQueryRequestScope>,
) -> (Record, u64, bool) {
    let outcome = if let Some(scope) = scope {
        workspace
            .advancement_owner()
            .with_request_advancement(scope, |phase| {
                match run.advance_frontier_once(&phase, members, workspace) {
                    Ok(super::super::super::WorthQueryWorkflowAdvanceStep::Advanced) => {
                        TransitionOutcome::Success(run)
                    }
                    Ok(super::super::super::WorthQueryWorkflowAdvanceStep::Deferred(
                        conditional,
                    )) => TransitionOutcome::Deferred(
                        crate::domain_installation::WorthQueryDeferredWorkflowStage {
                            run,
                            conditional,
                        },
                    ),
                    Err(denial) => run.outcome_from_denial(denial),
                }
            })
            .unwrap_or_else(|cause| {
                TransitionOutcome::Denied(domain::WorthQueryWorkflowAdvanceDenial::new(
                    domain::WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause),
                    Default::default(),
                ))
            })
    } else {
        run.advance_admitted_frontier(members, workspace)
    };
    match outcome {
        TransitionOutcome::Success(run) => (
            record(None, run.counters(), run.receipts(), &[]),
            run.counters().computation_charged_work,
            false,
        ),
        TransitionOutcome::Failed(denial) => (
            record(
                Some(denial.kind().clone()),
                denial.counters(),
                denial.completed_stage_receipts(),
                denial.executed_effects(),
            ),
            denial.counters().computation_charged_work,
            true,
        ),
        TransitionOutcome::Denied(denial) => (
            record(
                Some(denial.kind().clone()),
                denial.counters(),
                denial.completed_stage_receipts(),
                denial.executed_effects(),
            ),
            denial.counters().computation_charged_work,
            false,
        ),
        _ => panic!("unexpected fixture transition"),
    }
}
#[test]
fn seeded_frontiers_are_identical_at_serial_one_two_and_four_workers() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for seed in [1, 7, 29] {
        for failure in std::iter::once(None)
            .chain((0..3).flat_map(|at| (1..=7).map(move |mode| Some((at, mode)))))
        {
            let expected = observe(request_execution, seed, failure, Policy::width(0), &probe);
            // Preparation fails before its member computes. Compute failures,
            // panic and result capacity settle through that member. Apply never
            // refunds the computation of the complete prepared frontier.
            let charged_members = failure.map_or(3, |(at, mode)| match mode {
                1 => at,
                2 | 6 | 7 => at + 1,
                _ => 3,
            });
            assert_eq!(
                expected.1,
                COSTS[..charged_members].iter().sum::<u64>(),
                "seed={seed} failure={failure:?}"
            );
            for workers in [1, 2, 4] {
                let actual = observe(
                    request_execution,
                    seed,
                    failure,
                    Policy::width(workers),
                    &probe,
                );
                assert_eq!(
                    actual, expected,
                    "workers={workers} seed={seed} failure={failure:?}"
                );
            }
        }
    }
}
#[test]
fn reversed_completion_still_applies_canonical_effects_and_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    // The independent owner quote funds this successful journey. Work refusal
    // has its own finite-boundary court.
    probe.take_computes();
    let (expected, _) = worth_execution::ExecutionWorkCeiling::new(None)
        .run_serial(&host_request, || {
            observe(request_execution, 29, None, Policy::width(0), &probe)
        })
        .unwrap();
    // Reset observations between runs: any purity mutant must prove order
    // dependence, not simply a counter carried over from a previous run.
    probe.take_computes();
    assert_eq!(expected.0.outcome, None);
    let repeated = observe(request_execution, 29, None, Policy::width(0), &probe);
    assert_eq!(repeated.0.outcome, None);
    assert_eq!(repeated, expected, "two reset serial runs agree");
    assert_eq!(
        expected
            .0
            .receipts
            .iter()
            .map(|receipt| receipt.0.as_str())
            .collect::<Vec<_>>(),
        ["start", "left", "middle", "right"]
    );

    for workers in [1, 2] {
        probe.take_computes();
        assert_eq!(
            observe(request_execution, 29, None, Policy::width(workers), &probe),
            expected,
            "funded original grant at width {workers}"
        );
    }

    probe.take_computes();
    probe.reorder();
    let actual = observe(request_execution, 29, None, Policy::width(4), &probe);
    assert_eq!(actual.0.outcome, None);
    probe.assert_schedule_completed();
    assert_eq!(actual, expected);
    let completions = probe.take_computes();
    let position = |name| {
        completions
            .iter()
            .position(|(stage, _)| stage == name)
            .unwrap()
    };
    assert!(
        position("middle") < position("left"),
        "rendezvous forces genuine overlap"
    );
}
#[test]
fn owner_failure_does_not_refund_settled_compute() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    let mut input = members(7, Some((0, 4)));
    input[2].1 = domain::WorthQueryWorkflowValue::Text("9:2:41".into());
    let mut workspace = phased_workflow::phased_workspace(request_execution, "width-oracle");
    let (_, charged, failed) = observe_members(start(&mut workspace), input, &mut workspace, None);
    assert!(failed);
    assert_eq!(charged, COSTS.iter().sum::<u64>());
}

#[test]
fn empty_frontier_keeps_shape_refusal_and_singleton_uses_the_same_metered_map() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let mut workspace = phased_workflow::phased_workspace(request_execution, "empty-frontier");
    let (empty, work, failed) =
        observe_members(start(&mut workspace), Vec::new(), &mut workspace, None);
    assert_eq!(
        empty.outcome,
        Some(domain::WorthQueryWorkflowAdvanceDenialKind::ParallelFrontierShape)
    );
    assert_eq!(work, 0);
    assert!(!failed);
    let mut expected = None;
    for workers in [0, 1, 2, 4] {
        let mut workspace = Policy::width(workers).workspace(request_execution, &probe);
        let run = start(&mut workspace)
            .advance(
                "left",
                domain::WorthQueryWorkflowValue::Text("29:0:3".into()),
                &mut workspace,
            )
            .unwrap();
        assert_eq!(run.counters().computation_charged_work, 3);
        let actual = record(None, run.counters(), run.receipts(), &[]);
        if let Some(expected) = &expected {
            assert_eq!(&actual, expected);
        } else {
            expected = Some(actual);
        }
    }
}
