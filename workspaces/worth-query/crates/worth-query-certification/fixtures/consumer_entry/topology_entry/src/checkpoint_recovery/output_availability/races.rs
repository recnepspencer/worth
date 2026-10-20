//! Real completion interleaves after the consumer's native candidate preparation.
//! SourceBoundary: only this file uses the existing receiver-scoped pause control.
use super::*;
use journey::{controls, finish};
use std::time::Duration;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
    WorthQueryOutputSettlementPosture, WorthQueryProgramOutputAdvanceDenial,
    WorthQueryProgramOutputOperationAdvance, WorthQueryRequiredOutputPreparationDenial,
};

pub(super) fn complete_geometry(
    app: &installation::App,
    request: &journey::Request<'_, '_, '_>,
    equal: bool,
) {
    let before = request.retain_current_read().unwrap();
    let prior_events = equal.then(
        worth_query_host::facade::primary_graph::advancement_observations_on_this_thread_for_test,
    );
    settle_geometry(app, request, equal);
    if equal {
        assert_eq!(
            before.selected_commit(),
            request.retain_current_read().unwrap().selected_commit(),
            "sequential equal-input completion changes no World business commit; this completion events: {:?}; provider contacts: {}; prior event count: {}",
            worth_query_host::facade::primary_graph::advancement_observations_on_this_thread_for_test()
                .into_iter().flat_map(|record| record.applied).collect::<Vec<_>>(),
            crate::producer::provider_contacts(),
            prior_events.as_ref().map_or(0, Vec::len)
        );
    }
}
fn settle_geometry(app: &installation::App, request: &journey::Request<'_, '_, '_>, equal: bool) {
    let mut owner = request
        .demand(PlanarOutputDemand::new("measure-a"))
        .controls(controls())
        .start_in_program::<Program, GeometryRoot>(app)
        .unwrap();
    let mut completed = None;
    for _ in 0..256 {
        match owner.advance(request) {
            Ok(
                WorthQueryApplicationOutputDemandProgress::Pending
                | WorthQueryApplicationOutputDemandProgress::Progressed,
            ) => {}
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(value)) => {
                completed = Some(value);
                break;
            }
            // The earlier serialized publisher may change the selected Product.
            // Retry only that native refusal, on this SAME original demand owner.
            Err(WorthQueryApplicationOutputDemandDenial::Demand(stop))
                if stop.kind() == WorthQueryOutputDemandDenialKind::PublicationStale => {}
            Err(stop) => panic!("actual producer refusal: {stop:?}"),
        }
    }
    let completed = completed.expect("actual root producer completes");
    if equal {
        assert_eq!(
            completed.posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(completed.producer_contacts_in_this_demand(), 0);
        assert!(completed.application_commit_receipt().is_none());
    }
    owner.close();
}

#[test]
fn first_and_equal_completion_serialize_behind_original_publisher_then_wake_pending() {
    let _guard = checkpoint_recovery_test_guard();
    for equal in [false, true] {
        let mut app = installation::install();
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        let previous = if equal {
            complete_geometry(&app, &request, false);
            let ready = journey::quantity(&app, &request);
            assert_eq!(
                ready
                    .read_current(&request, read::Read(publication::output_key("population")))
                    .unwrap()
                    .rows()[0]
                    .measure,
                Measure::Complete(11)
            );
            let source = request
                .query(PlanarRead {
                    body_key: "measure-b".into(),
                })
                .execute()
                .unwrap();
            request
                .advanced()
                .mutate(PlanarSourceAdjustment {
                    scope_key: "measure-b".into(),
                    replacement_y: length(11),
                })
                .expect_source(source.observed_sources()[0].clone())
                .idempotency(&0x91742702u64)
                .execute_performed::<Program, GeometryRoot>(
                    &app,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            Some(ready)
        } else {
            None
        };
        let original = request
            .demand(producer::Demand("population".into()))
            .controls(controls())
            .retain_program_outputs::<Program, QuantityRoot>(&app)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        let pause = app.pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
        let settled = std::thread::scope(|threads| {
            let pause = &pause;
            let app = &app;
            let other = threads.spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    assert!(
                        pause.wait_until_reached(Duration::from_secs(10)),
                        "original prepared candidate holds its actual branch lane"
                    );
                    let blocked = app.observe_application_branch_commit_contention_for_test();
                    let (done, completion) = std::sync::mpsc::channel();
                    std::thread::scope(|workers| {
                        let upstream = workers.spawn(move || {
                            let (scope, principal) = authenticate(app);
                            let fresh = app.request(&principal, &scope);
                            settle_geometry(app, &fresh, equal);
                            done.send(()).unwrap();
                        });
                        let reached = blocked.wait_until_blocked(Duration::from_secs(10));
                        let unfinished = matches!(
                            completion.try_recv(),
                            Err(std::sync::mpsc::TryRecvError::Empty)
                        );
                        // Release before assertions so a failing witness cannot
                        // strand a genuine queued native producer during unwind.
                        pause.release();
                        assert!(
                            reached,
                            "queued REAL first/stable producer reaches actual Mutex WouldBlock"
                        );
                        assert!(
                            unfinished,
                            "queued producer cannot finish while original holds lane"
                        );
                        upstream.join().unwrap();
                        completion.recv().unwrap();
                    });
                }));
                pause.release();
                result.unwrap();
            });
            let settled = finish!(original, &request);
            other.join().unwrap();
            settled
        });
        // Use the original parent-owned pinned observation honestly as history.
        // Upstream may already have completed before this worker is scheduled.
        let historical = request
            .at(settled.settlement().observation())
            .query(read::Read(publication::output_key("population")))
            .execute()
            .unwrap();
        assert_eq!(historical.rows()[0].count, 1);
        assert_eq!(historical.rows()[0].measure, Measure::Pending);
        assert!(
            settled
                .read_current(&request, read::Read(publication::output_key("population")))
                .is_err(),
            "real later completion invalidates the earlier original Pending owner"
        );
        let renewed = journey::quantity(&app, &request);
        let value = renewed
            .read_current(&request, read::Read(publication::output_key("population")))
            .unwrap();
        assert_eq!(value.rows()[0].count, 1);
        assert_eq!(value.rows()[0].measure, Measure::Complete(11));
        let renewed_id = renewed
            .settlement()
            .root_settlement()
            .outputs_of::<binding::PreserveOutputs>()
            .unwrap()
            .entity::<binding::Preserved>()
            .unwrap()
            .entity_id();
        let original_id = if equal {
            settled
                .settlement()
                .root_settlement()
                .outputs_of::<binding::PreserveOutputs>()
                .unwrap()
                .entity::<binding::Preserved>()
                .unwrap()
                .entity_id()
        } else {
            settled
                .settlement()
                .root_settlement()
                .outputs_of::<binding::InitialOutputs>()
                .unwrap()
                .entity::<binding::Created>()
                .unwrap()
                .entity_id()
        };
        assert_eq!(original_id, renewed_id);
        drop((historical, value));
        settled
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        renewed
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        if let Some(previous) = previous {
            previous
                .close(&request)
                .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        }
        drop((principal, scope));
        app.close_conditional_runtime().unwrap();
    }
}

#[test]
fn live_different_source_and_missing_source_refuse_instead_of_pending() {
    let _guard = checkpoint_recovery_test_guard();
    for key in ["measure-b", "missing-source"] {
        let mut app = installation::install_selected(key);
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        let mut operation = request
            .demand(producer::Demand("population".into()))
            .controls(controls())
            .retain_program_outputs::<Program, QuantityRoot>(&app)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        loop {
            operation = match operation.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
                    assert_source_refusal(&denial, key);
                    operation
                        .close(&request)
                        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    assert_source_refusal(terminal.denial(), key);
                    terminal
                        .close(&request)
                        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Settled(_) => panic!(
                    "selection input cannot grant source identity or turn missing into Pending"
                ),
            };
        }
        assert!(
            request
                .query(read::Read(publication::output_key("population")))
                .execute()
                .is_err(),
            "refusal leaves no published quantity"
        );
        drop((principal, scope));
        app.close_conditional_runtime().unwrap();
    }
}

mod interruption;
mod nested;

fn assert_source_refusal(denial: &WorthQueryProgramOutputAdvanceDenial, key: &str) {
    let kind = match denial {
        WorthQueryProgramOutputAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::DemandExecution(stop),
        ) => stop.kind(),
        WorthQueryProgramOutputAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Demand(stop),
            ),
        ) => stop.kind(),
        _ => panic!("the source observer must own this refusal: {denial:?}"),
    };
    if key == "measure-b" {
        assert_eq!(kind, WorthQueryOutputDemandDenialKind::ForeignSource);
    } else {
        assert!(
            matches!(kind, WorthQueryOutputDemandDenialKind::SourceScope(_)),
            "missing native scope must remain an actual source-scope refusal: {denial:?}"
        );
    }
}
