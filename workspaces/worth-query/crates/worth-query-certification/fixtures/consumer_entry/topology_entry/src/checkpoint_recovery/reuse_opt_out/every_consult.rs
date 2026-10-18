//! Cancellation is observed by the complete retained producer selection.
use super::*;
use worth_execution::with_request_consult_observer_for_test;

#[test]
fn whole_selection_preserves_cancellation_at_every_consult() {
    let _guard = checkpoint_recovery_test_guard();
    let _observation = primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let application = install_no_reuse(None);
    let (scope, principal) = support::authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut initial = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(&application)
        .unwrap();
    let first = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the real producer publishes the retained candidate");
    drop((first, initial));
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let run = |cancel_at: Option<usize>| {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
        application
            .with_application_advancement(&scope, |phase| {
                with_request_consult_observer_for_test(
                    move |at| {
                        if Some(at) == cancel_at {
                            cancellation.cancel();
                        }
                    },
                    || {
                        application
                            .select_output_producer::<PlanarFinalOutputFamily>(
                                &phase,
                                &source.observed_sources()[0],
                                "planar-final",
                                std::num::NonZeroUsize::new(4096),
                            )
                            .map(|selected| selected.applicability().lifecycle())
                            .map_err(|denied| denied.kind())
                    },
                )
            })
            .expect("the host opens before the selection observer is installed")
    };
    let (healthy, count) = run(None);
    assert_eq!(healthy, Ok(WorthQueryProducerLifecyclePosture::Preserve));
    assert!(count > 0);
    for at in 1..=count + 1 {
        let (answer, _) = run(Some(at));
        assert_eq!(
            answer,
            if at > count {
                healthy.clone()
            } else {
                Err(primary_graph::WorthQueryOutputDemandDenialKind::Cancelled)
            },
            "selection consult {at} of {count}"
        );
    }
}
