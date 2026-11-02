//! Cancel the original request after the first real rule consumes staged roots.
use super::*;
use std::sync::atomic::AtomicBool;
use worth_query_host::facade::admission::authenticated_principal::*;

pub(super) fn reject(
    world: &host::Host,
    provider: &Arc<prefix::Calls>,
    rules: &Arc<rule::Calls>,
    suspended: WorthQuerySuspendedGeneratedOutput,
) -> WorthQuerySuspendedGeneratedOutput {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let reached = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&reached);
    rules.records.lock().unwrap().clear();
    let builds = provider.builds.load(Ordering::SeqCst);
    let updates = provider.updates.load(Ordering::SeqCst);
    *rules.after_first_observation.lock().unwrap() = Some(Box::new(move || {
        observed.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    let completed = reconstruction::complete(world, suspended, "taken-generated");
    let failure = match world.restore_generated_output(completed, &request) {
        Err(failure) => failure,
        Ok(_) => panic!("original cancellation after Source observation must stop restoration"),
    };
    assert!(
        reached.load(Ordering::SeqCst),
        "cancellation follows a real rule query of already prepared proposed roots"
    );
    assert_eq!(provider.builds.load(Ordering::SeqCst), builds);
    assert_eq!(provider.updates.load(Ordering::SeqCst), updates);
    let WorthQueryGeneratedOutputRestorationFailure::Rejected { suspended, cause } = failure else {
        panic!("stopped preparation retains suspension custody before any cutover")
    };
    assert!(
        matches!(
            cause,
            WorthQueryGeneratedOutputRestorationFailureCause::ExecutionControlStopped(
                WorthQueryProviderSessionControlStopKind::Cancelled
            )
        ),
        "original typed cancellation survives: {cause:?}"
    );
    let records = rules.records.lock().unwrap();
    assert_eq!(
        records.len(),
        1,
        "original cancellation prevents the second rule callback"
    );
    assert!(
        records[0].before.is_none(),
        "the logical reuse grant never opens unavailable committed Source"
    );
    suspended
}
