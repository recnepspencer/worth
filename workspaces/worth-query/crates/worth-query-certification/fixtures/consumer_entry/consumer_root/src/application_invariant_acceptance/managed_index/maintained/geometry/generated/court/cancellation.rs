//! Original suspension cancellation follows a real committed/proposed Source query.
use super::*;
use std::sync::atomic::AtomicBool;
pub(super) fn suspension(
    world: &host::Host,
    rules: &Arc<rule::Calls>,
    source: WorthQueryObservedSource<source::Query>,
    scopes: [((u32, u64, u32), &str); 2],
) {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let reached = Arc::new(AtomicBool::new(false));
    let observed = reached.clone();
    rules.records.lock().unwrap().clear();
    *rules.after_first_observation.lock().unwrap() = Some(Box::new(move || {
        observed.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let failure =
        match selected.suspend_current_generated_output::<producer::Producer>(&request, source) {
            Err(failure) => failure,
            Ok(_) => panic!("original suspension cancellation must stop before publication"),
        };
    assert_eq!(
        failure.execution_control_stop(),
        Some(WorthQueryProviderSessionControlStopKind::Cancelled),
        "preserve the original typed cause: {failure:?}"
    );
    assert!(reached.load(Ordering::SeqCst));
    let rows = rules.records.lock().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "the original cancellation prevents the second scope callback"
    );
    let row = &rows[0];
    let directory = scopes
        .iter()
        .find(|(scope, _)| *scope == row.scope)
        .unwrap()
        .1;
    assert_eq!(
        values(row.source_after.as_ref().unwrap()),
        expected(directory, 10)
    );
    assert_eq!(
        values(row.source_before.as_ref().unwrap()),
        expected(directory, 10)
    );
}
