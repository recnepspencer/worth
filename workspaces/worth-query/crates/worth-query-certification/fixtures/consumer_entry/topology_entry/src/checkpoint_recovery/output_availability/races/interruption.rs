//! Cancellation is issued after admission at the actual source-fact comparison.
use super::*;
#[test]
fn original_cancellation_during_source_comparison_survives_and_returns_retry_owner() {
    let _guard = checkpoint_recovery_test_guard();
    let mut app = installation::install();
    let (valid_scope, principal) = authenticate(&app);
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
    let request = app.request(&principal, &scope);
    let original = request
        .demand(producer::Demand("population".into()))
        .controls(controls())
        .retain_program_outputs::<Program, QuantityRoot>(&app)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    let pause = app.pause_before_source_fact_comparison_for_test(NonZeroUsize::MIN);
    let returned = std::thread::scope(|threads| {
        let pause = &pause;
        let cancellation = &cancellation;
        let trigger = threads.spawn(move || {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            cancellation.cancel();
            pause.release();
            assert!(reached, "actual source-fact comparison reached");
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
                        _ => panic!(
                            "actual comparison must retain original control cause: {denial:?}"
                        ),
                    };
                    assert_eq!(
                        kind,
                        WorthQueryOutputDemandDenialKind::Cancelled,
                        "cancellation must never become WorkBudgetExceeded"
                    );
                    break operation;
                }
                WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                    panic!("cancelled comparison cannot publish")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(stop) => panic!(
                    "cancelled comparison must return original retry owner: {:?}",
                    stop.denial()
                ),
            };
        };
        trigger.join().unwrap();
        returned
    });
    drop(pause);
    let fresh = app.request(&principal, &valid_scope);
    let settled = finish!(returned, &fresh);
    let value = settled
        .read_current(&fresh, read::Read(publication::output_key("population")))
        .unwrap();
    assert_eq!(value.rows()[0].count, 1);
    assert_eq!(value.rows()[0].measure, Measure::Pending);
    drop(value);
    settled
        .close(&fresh)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    drop((principal, valid_scope, scope));
    app.close_conditional_runtime().unwrap();
}
