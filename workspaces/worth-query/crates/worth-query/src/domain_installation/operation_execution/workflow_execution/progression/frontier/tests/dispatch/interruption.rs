//! Entry interruption equality and constructed in-compute prefix safety.
use super::*;
use domain::WorthQueryWorkflowAdvanceDenialKind as Denial;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource as CancellationSource;
#[test]
fn predispatch_cancellation_is_identical_at_every_first_compute_position_and_width() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    for first in 0..3 {
        let mut expected = None;
        for workers in [0, 1, 2, 4] {
            probe.take_computes();
            let source = CancellationSource::new();
            source.cancel();
            let scope = WorthQueryRequestScope::new(
                std::time::Instant::now() + std::time::Duration::from_secs(60),
                source.token(),
            );
            let mut input = members(7, None);
            // Earlier members declare empty, zero-work tasks. The varied member
            // is the first that would checkpoint if entry were not interrupted.
            for (rank, member) in input.iter_mut().enumerate().take(first) {
                member.1 = domain::WorthQueryWorkflowValue::Text(format!("{}:9:0", 7 + rank));
            }
            let mut workspace = Policy::width(workers).workspace(&probe);
            let actual =
                observe_members(start(&mut workspace), input, &mut workspace, Some(&scope));
            assert_eq!(
                actual.0.outcome,
                Some(Denial::ExecutionRequest(
                    worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                        worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
                    )
                ))
            );
            assert_eq!(actual.1, 0);
            assert!(!actual.2);
            assert!(probe.take_computes().is_empty());
            assert!(actual.0.effects.is_empty());
            assert_eq!(actual.0.receipts.len(), 0);
            if let Some(expected) = &expected {
                assert_eq!(&actual, expected);
            } else {
                expected = Some(actual);
            }
        }
    }
}
#[test]
fn cancellation_during_middle_and_last_compute_obeys_each_constructed_schedule() {
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let successful = observe(7, None, Policy::width(0), &probe).0;
    for at in [1, 2] {
        for workers in [0, 1, 2, 4] {
            for hold_earlier in [false, true] {
                probe.take_computes();
                // A one-worker posture cannot hold an earlier member until a
                // later member runs. Its held schedule necessarily degenerates
                // to canonical release; parallel postures force true overlap.
                let held = hold_earlier && workers >= 2;
                let source = CancellationSource::new();
                probe.cancel_at_checkpoint(source.clone(), phased_workflow::MEMBERS[at], held);
                let scope = WorthQueryRequestScope::new(
                    std::time::Instant::now() + std::time::Duration::from_secs(60),
                    source.token(),
                );
                let mut workspace = Policy::width(workers).workspace(&probe);
                let run = start(&mut workspace);
                let (actual, charged, failed) =
                    observe_members(run, members(7, Some((at, 8))), &mut workspace, Some(&scope));
                probe.assert_schedule_completed();
                let boundary = if held { 0 } else { at };
                assert_eq!(
                    actual.outcome,
                    Some(Denial::ComputationCancelled {
                        stage_identity: Some(phased_workflow::MEMBERS[boundary].into()),
                    }),
                    "at={at} workers={workers} held={held}"
                );
                assert!(!failed);
                assert_eq!(
                    actual
                        .receipts
                        .iter()
                        .map(|r| r.0.as_str())
                        .collect::<Vec<_>>(),
                    std::iter::once("start")
                        .chain(phased_workflow::MEMBERS[..boundary].iter().copied())
                        .collect::<Vec<_>>()
                );
                assert_eq!(actual.effects.len(), boundary);
                actual.assert_owner_prefix_of(&successful);
                // Checkpoint rejects before charging the held earlier member;
                // otherwise all predecessors finish and the interrupter charges
                // exactly its first unit. Suffix work is never settled.
                let expected_charge = if held {
                    0
                } else {
                    COSTS[..at].iter().sum::<u64>() + 1
                };
                assert_eq!(charged, expected_charge);
                let events = probe.take_computes();
                assert!(events
                    .iter()
                    .any(|(member, work)| member == phased_workflow::MEMBERS[at] && *work == 1));
                if held {
                    assert!(events
                        .iter()
                        .any(|(member, work)| member == "left" && *work == 0));
                }
            }
        }
    }
}
