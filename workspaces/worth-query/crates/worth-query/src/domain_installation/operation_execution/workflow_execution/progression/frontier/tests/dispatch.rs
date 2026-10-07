//! Actual authority placement oracle, with no schedule hook in Query.
use super::differential::{record_billed as record, Record};
use super::{domain, phased_workflow, start};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_proof::TransitionOutcome;
#[path = "dispatch/interruption.rs"]
mod interruption;
#[path = "dispatch/stops.rs"]
mod stops;
const COSTS: [u64; 3] = [3, 17, 41];

fn lease_request(workers: usize, memory: u64, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), memory, work),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
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
    seed: u64,
    failure: Option<(usize, u64)>,
    request: ExecutionRequest<'_, '_>,
) -> (Record, u64, bool) {
    let mut workspace = phased_workflow::phased_workspace("width-oracle");
    observe_members(
        start(&mut workspace),
        members(seed, failure),
        &mut workspace,
        request,
    )
}
fn observe_members(
    run: super::Run,
    members: Vec<(String, domain::WorthQueryWorkflowValue)>,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    request: ExecutionRequest<'_, '_>,
) -> (Record, u64, bool) {
    match run.advance_admitted_frontier(members, workspace, request) {
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
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for seed in [1, 7, 29] {
        for failure in std::iter::once(None)
            .chain((0..3).flat_map(|at| (1..=7).map(move |mode| Some((at, mode)))))
        {
            let serial = super::request::workflow_request();
            let expected = observe(seed, failure, ExecutionRequest::serial(&serial));
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
                let lease = probe
                    .authority()
                    .request_lease(lease_request(workers, 64 * 1024 * 1024, 1024))
                    .unwrap();
                let actual = observe(seed, failure, ExecutionRequest::leased(&lease));
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
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let serial = super::request::workflow_request();
    probe.take_computes();
    let expected = observe(29, None, ExecutionRequest::serial(&serial));
    // Reset observations between runs: any purity mutant must prove order
    // dependence, not simply a counter carried over from a previous run.
    probe.take_computes();
    assert_eq!(
        observe(29, None, ExecutionRequest::serial(&serial)),
        expected,
        "two reset serial runs agree"
    );
    assert_eq!(expected.0.outcome, None);
    assert_eq!(
        expected
            .0
            .receipts
            .iter()
            .map(|receipt| receipt.0.as_str())
            .collect::<Vec<_>>(),
        ["start", "left", "middle", "right"]
    );

    probe.take_computes();
    probe.reorder();
    let lease = probe
        .authority()
        .request_lease(lease_request(4, 64 * 1024 * 1024, 1024))
        .unwrap();
    let actual = observe(29, None, ExecutionRequest::leased(&lease));
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
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    let serial = super::request::workflow_request();
    let mut input = members(7, Some((0, 4)));
    input[2].1 = domain::WorthQueryWorkflowValue::Text("9:2:41".into());
    let mut workspace = phased_workflow::phased_workspace("width-oracle");
    let (_, charged, failed) = observe_members(
        start(&mut workspace),
        input,
        &mut workspace,
        ExecutionRequest::serial(&serial),
    );
    assert!(failed);
    assert_eq!(charged, COSTS.iter().sum::<u64>());
}

#[test]
fn empty_frontier_keeps_shape_refusal_and_singleton_uses_the_same_metered_map() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let serial = super::request::workflow_request();
    let mut workspace = phased_workflow::phased_workspace("empty-frontier");
    let (empty, work, failed) = observe_members(
        start(&mut workspace),
        Vec::new(),
        &mut workspace,
        ExecutionRequest::serial(&serial),
    );
    assert_eq!(
        empty.outcome,
        Some(domain::WorthQueryWorkflowAdvanceDenialKind::ParallelFrontierShape)
    );
    assert_eq!(work, 0);
    assert!(!failed);
    let mut expected = None;
    for workers in [0, 1, 2, 4] {
        let lease = (workers != 0).then(|| {
            probe
                .authority()
                .request_lease(lease_request(workers, 64 * 1024 * 1024, 1024))
                .unwrap()
        });
        let request = lease
            .as_ref()
            .map_or(ExecutionRequest::serial(&serial), ExecutionRequest::leased);
        let mut workspace = phased_workflow::phased_workspace("singleton-width");
        let run = start(&mut workspace)
            .advance(
                "left",
                domain::WorthQueryWorkflowValue::Text("29:0:3".into()),
                &mut workspace,
                request,
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
