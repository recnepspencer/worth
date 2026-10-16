use super::{PlatformPulseQueryLifecycle, PlatformPulseQueryLifecycleDenial};
use worth_ui::facade::query_binding::{
    UiProjectionObservation, WorthUiScalarProjectionSourceRecord, WorthUiStatusSourceOwner,
};

#[test]
fn committed_query_update_cannot_report_clean_close_before_presentation() {
    let owner = WorthUiStatusSourceOwner::install().expect("install authored Query owner");
    let (_, initial) = owner
        .initial_projection()
        .expect("issue initial Query fact");
    let mut lifecycle = PlatformPulseQueryLifecycle::new(owner, initial);
    let UiProjectionObservation::ApplicationScalar(initial) = lifecycle
        .issue_initial()
        .expect("issue the initial observation")
    else {
        panic!("the authored Query owner must issue an application scalar fact");
    };
    lifecycle
        .admit_publication(initial)
        .expect("admit the exact initial fact");
    let pending = lifecycle
        .advance(
            WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("first status revision"),
        )
        .expect("Query commits and issues the next fact");
    assert!(matches!(
        pending,
        UiProjectionObservation::ApplicationScalar(_)
    ));
    assert!(matches!(
        lifecycle.close(),
        Err(PlatformPulseQueryLifecycleDenial::UnsettledPublicationAtClose)
    ));
}
