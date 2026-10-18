//! Actual outside-lane enrollment must not replace the validated image.
//! SourceBoundary: scheduling uses only this receiver's closed native controls.
use super::*;
#[test]
fn outside_lane_source_enrollment_after_full_validation_refuses_original_image() {
    let _guard = checkpoint_recovery_test_guard();
    let (mut app, enrollment) = installation::install_with_enrollment_window();
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    complete_geometry(&app, &request, false);
    let initial = journey::quantity(&app, &request);
    let summary = journey::quantity_scope(&app, &request, "summary");
    assert_eq!(
        summary
            .read_current(&request, read::Read(publication::output_key("summary")))
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
        .idempotency(&0x91742703u64)
        .execute_performed::<Program, GeometryRoot>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let pending = journey::quantity(&app, &request);
    let outer = request
        .query(read::Read(publication::output_key("population")))
        .execute()
        .unwrap();
    assert_eq!(outer.rows()[0].measure, Measure::Pending);
    let outer_identity = pending
        .settlement()
        .root_settlement()
        .outputs_of::<binding::PreserveOutputs>()
        .unwrap()
        .entity::<binding::Preserved>()
        .unwrap()
        .entity_id();
    let original = request
        .demand(producer::Demand("summary".into()))
        .controls(controls())
        .retain_program_outputs::<Program, QuantityRoot>(&app)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    let pause = app.pause_after_source_publication_validation_for_test(NonZeroUsize::MIN);
    let returned = std::thread::scope(|threads| {
        let (refused, original_returned) = std::sync::mpsc::channel();
        let pause = &pause;
        let app = &app;
        let other = threads.spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert!(
                    pause.wait_until_reached(Duration::from_secs(10)),
                    "original owning full validation reached"
                );
                let computing = enrollment.arm();
                std::thread::scope(|workers| {
                    let enroller = workers.spawn(|| {
                        let (scope, principal) = authenticate(app);
                        let fresh = app.request(&principal, &scope);
                        // This is a real different placed population using the SAME
                        // installed collection/handler. Its source Query observes
                        // measure-b for the first time and enrolls that native slot.
                        let owner = journey::quantity_scope(app, &fresh, "population-b");
                        let value = owner
                            .read_current(
                                &fresh,
                                read::Read(publication::output_key("population-b")),
                            )
                            .unwrap();
                        assert_eq!(value.rows()[0].count, 1);
                        assert_eq!(value.rows()[0].measure, Measure::Pending);
                        drop(value);
                        owner
                            .close(&fresh)
                            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
                    });
                    let enrolled = computing.wait_until_reached(Duration::from_secs(10));
                    // Release original even on a failed setup witness. The REAL
                    // second consumer is parked in reader-free COMPUTE, after
                    // enrollment/projection unlock and BEFORE commit preparation
                    // resolves idempotency under the branch lane.
                    pause.release();
                    if !enrolled {
                        computing.release();
                    }
                    assert!(
                        enrolled,
                        "actual second consumer reached reader-free compute after enrollment"
                    );
                    let rejected = original_returned.recv_timeout(Duration::from_secs(10));
                    computing.release();
                    rejected.expect("original image must refuse before enroller publishes");
                    enroller.join().unwrap();
                });
            }));
            pause.release();
            result.unwrap();
        });
        let mut operation = original;
        let returned = loop {
            operation = match operation.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
                    let kind = match &denial {
                        WorthQueryProgramOutputAdvanceDenial::Outputs(
                            WorthQueryRequiredOutputPreparationDenial::DemandExecution(stop),
                        ) => stop.kind(),
                        WorthQueryProgramOutputAdvanceDenial::Outputs(
                            WorthQueryRequiredOutputPreparationDenial::Demand(
                                WorthQueryApplicationOutputDemandDenial::Demand(stop),
                            ),
                        ) => stop.kind(),
                        _ => panic!("native image restriction owns refusal: {denial:?}"),
                    };
                    assert_eq!(kind, WorthQueryOutputDemandDenialKind::PublicationStale);
                    refused.send(()).unwrap();
                    break operation;
                }
                WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                    panic!("same outer heads do not permit replacing original validated image")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(stop) => {
                    panic!("original retry custody lost: {:?}", stop.denial())
                }
            };
        };
        other.join().unwrap();
        returned
    });
    let receipts = pause.observations();
    assert_eq!(
        receipts.len(),
        2,
        "owning validation images before and after REAL enrollment"
    );
    let (before, after) = (&receipts[0], &receipts[1]);
    assert!(before.native_commit.is_some() && before.native_position.is_some());
    assert_eq!(
        (before.root_id, before.native_commit, before.native_position),
        (after.root_id, after.native_commit, after.native_position),
        "outside-lane enrollment changes no native physical coordinate"
    );
    assert_eq!(before.outer_publications.len(), 1);
    assert!(!before.outer_publications[0].is_empty());
    assert_eq!(
        before.outer_publications, after.outer_publications,
        "exact outer bindings/partitions/native occurrence/generation/slot unchanged"
    );
    assert_ne!(
        before.topology_generation, after.topology_generation,
        "real admitted new source enrollment changes the native companion image"
    );
    eprintln!(
        "native availability image: original retry owner returned; physical={:?}; outer={:?}; topology {:?}->{:?}",
        (before.root_id, before.native_commit, before.native_position),
        before.outer_publications, before.topology_generation, after.topology_generation
    );
    drop(pause);
    // Finish actual upstream separately, renew the intermediate and retry the
    // ORIGINAL refused summary owner. No enrollment is a completion certificate.
    // This original required custody remains live, so native queue progression
    // may also commit its consumer. The separate CLOSED-delivery test owns the
    // strict whole-World no-business-commit oracle; this still requires actual
    // StableReused, zero root contacts and no root producer receipt.
    settle_geometry(&app, &request, true);
    let complete = journey::quantity(&app, &request);
    assert_eq!(
        outer_identity,
        complete
            .settlement()
            .root_settlement()
            .outputs_of::<binding::PreserveOutputs>()
            .unwrap()
            .entity::<binding::Preserved>()
            .unwrap()
            .entity_id()
    );
    let renewed = finish!(returned, &request);
    assert_eq!(
        renewed
            .read_current(&request, read::Read(publication::output_key("summary")))
            .unwrap()
            .rows()[0]
            .measure,
        Measure::Complete(11)
    );
    assert_eq!(outer.rows()[0].measure, Measure::Pending);
    drop((outer, source));
    for owner in [initial, summary, pending, complete, renewed] {
        owner
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    }
    drop((principal, scope));
    app.close_conditional_runtime().unwrap();
}
