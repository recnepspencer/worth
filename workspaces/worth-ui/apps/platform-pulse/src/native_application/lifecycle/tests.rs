//! CPU-only proof of the same Query gate native close uses before shell/watch teardown.
use super::close_query_for_shutdown;
use crate::query_source::{PlatformPulseQueryLifecycle, PlatformPulseQueryLifecycleDenial};
use worth_ui::facade::query_binding::{UiProjectionObservation, WorthUiStatusSourceOwner};

#[test]
fn native_shutdown_gate_preserves_the_whole_unpresented_query_owner() {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (_, initial) = owner.initial_projection().unwrap();
    let mut owner = Some(PlatformPulseQueryLifecycle::new(owner, initial));
    assert!(matches!(
        close_query_for_shutdown(&mut owner),
        Err(PlatformPulseQueryLifecycleDenial::OwnerNotLive)
    ));
    let UiProjectionObservation::ApplicationScalar(initial) = owner
        .as_mut()
        .expect("the whole initial lifecycle was restored")
        .issue_initial()
        .unwrap()
    else {
        panic!("real scalar fact expected");
    };
    assert!(matches!(
        close_query_for_shutdown(&mut owner),
        Err(PlatformPulseQueryLifecycleDenial::UnsettledPublicationAtClose)
    ));
    owner
        .as_mut()
        .expect("presentation-pending custody was restored")
        .admit_publication(initial)
        .unwrap();
    let closed = close_query_for_shutdown(&mut owner)
        .expect("only the genuine admitted presentation can close the original owner");
    assert!(closed.owner_terminal());
    assert!(
        owner.is_none(),
        "the source owner is consumed only by lawful close"
    );
}

#[test]
fn stopped_native_product_retains_query_and_watches_before_terminal_early_return() {
    let launch =
        crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration::from_arguments(
            Vec::<std::ffi::OsString>::new(),
        )
        .expect("canonical product inputs admit");
    let composition =
        crate::application::prepare_composition(&launch).expect("real CPU application preparation");
    let publisher =
        crate::lifecycle_observation_publication::PlatformPulseObservationPublisher::start()
            .unwrap();
    let (builder, mut runtime) = super::super::PlatformPulseApplicationRuntime::from_composition(
        composition,
        publisher,
        crate::visual_identity_execution::PlatformPulseVisualIdentityExecution::new(false),
    );
    runtime.fail(
        super::super::PlatformPulseTerminalError::FrameExecution(
            "pre-presentation stop fixture".to_owned(),
        ),
        Ok(()),
    );
    assert!(
        runtime.shutdown_product().is_none(),
        "no shell shutdown or runtime-closed evidence can be issued"
    );
    assert!(runtime.terminal.is_stopped());
    assert!(runtime.theme_watch.is_some());
    assert!(runtime.source_watch.is_some());
    assert!(runtime.query_watch.is_some());
    assert!(runtime.intent_watch.is_some());
    let lifecycle = runtime
        .query_lifecycle
        .as_mut()
        .expect("terminal early return cannot discard the original Query owner");
    let UiProjectionObservation::ApplicationScalar(initial) = lifecycle.issue_initial().unwrap()
    else {
        panic!("original initial context must remain available");
    };
    lifecycle.admit_publication(initial).unwrap();
    assert!(
        runtime.shutdown_product().is_none(),
        "this CPU preparation fixture has no native shell receipt"
    );
    assert!(
        runtime.query_lifecycle.is_none(),
        "fresh legal retry closed the actual Query owner"
    );
    assert!(runtime.source_watch.is_none());
    assert!(runtime.query_watch.is_none());
    assert!(runtime.intent_watch.is_none());
    drop(builder);
}
