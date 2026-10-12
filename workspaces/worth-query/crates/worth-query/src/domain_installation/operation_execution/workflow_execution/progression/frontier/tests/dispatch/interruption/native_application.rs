//! Caller cancellation cannot fund application of the computed prefix.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind;

#[test]
fn cancellation_during_compute_stops_native_application_and_keeps_settled_work() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let probe = phased_workflow::fold_executor::ComputeProbe::new();
    let successful = observe(&execution, 7, None, Policy::width(0), &probe).0;
    for at in [1, 2] {
        for workers in [0, 1, 2, 4] {
            for hold_earlier in [false, true] {
                probe.take_computes();
                let held = hold_earlier && workers >= 2;
                let source = CancellationSource::new();
                probe.cancel_at_checkpoint(source.clone(), phased_workflow::MEMBERS[at], held);
                let scope = WorthQueryRequestScope::new(
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
                    source.token(),
                );
                let mut workspace = Policy::width(workers).workspace(&execution, &probe);
                let run = start(&mut workspace);
                let (actual, charged, failed) =
                    observe_members(run, members(7, Some((at, 8))), &mut workspace, Some(&scope));
                probe.assert_schedule_completed();
                // A held first computation stops before an application exists.
                // Otherwise the first native application consults this same
                // cancelled caller instead of creating an unrelated request.
                let expected = if held {
                    Denial::ComputationCancelled {
                        stage_identity: Some("left".into()),
                    }
                } else {
                    Denial::StageExecutionRequest {
                        stage_identity: "left".into(),
                        cause: Err(WorthQueryProviderSessionControlStopKind::Cancelled),
                    }
                };
                assert_eq!(
                    actual.outcome,
                    Some(expected),
                    "at={at} workers={workers} held={held}"
                );
                assert!(!failed);
                assert_eq!(
                    actual
                        .receipts
                        .iter()
                        .map(|r| r.0.as_str())
                        .collect::<Vec<_>>(),
                    ["start"]
                );
                assert!(actual.effects.is_empty());
                actual.assert_owner_prefix_of(&successful);
                let expected_work = if held {
                    0
                } else {
                    COSTS[..at].iter().sum::<u64>() + 1
                };
                assert_eq!(charged, expected_work);
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
