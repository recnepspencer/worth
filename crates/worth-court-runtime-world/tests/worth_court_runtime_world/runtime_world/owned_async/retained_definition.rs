//! Fresh actual Signal/Bridge owners install meaning; DATA never grants a port.
use crate::owned_async::ordinary_context::fixture::{
    admit_managed, selected_fork, AsyncSourceCourt,
};

#[test]
fn owned_async_readmitted_definition_keeps_mandatory_and_opted_in_release_separate() {
    let mandatory = AsyncSourceCourt::managed(std::num::NonZeroU64::MIN);
    let selected = selected_fork(&mandatory);
    let owner = admit_managed(&mandatory, &selected);
    let witness = owner.retained_request().unwrap();
    let original = owner.request_handle();
    let refusal = mandatory
        .bridge
        .release_process_local_managed_request(owner, None)
        .err()
        .expect("a mandatory request cannot use the opted-in release route");
    assert!(matches!(refusal.cause(),
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(
            worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
                worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch))));
    let (owner, completion, _) = refusal.into_parts();
    assert_eq!(owner.request_handle(), original);
    assert!(completion.is_none());
    drop(owner);
    assert_eq!(
        mandatory
            .bridge
            .owned_async_active_request_count(&witness)
            .unwrap(),
        1,
        "a dropped admission must not cancel or retire mandatory work"
    );

    let opted_in = AsyncSourceCourt::process_local(std::num::NonZeroU64::MIN);
    let selected = selected_fork(&opted_in);
    let owner = admit_managed(&opted_in, &selected);
    let original = owner.request_handle();
    let refusal = opted_in
        .bridge
        .release_process_local_managed_request(owner, None)
        .err()
        .expect("release does not silently cancel active work");
    let (owner, _, _) = refusal.into_parts();
    assert_eq!(owner.request_handle(), original);
    opted_in
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .unwrap();
    let released = opted_in
        .bridge
        .release_process_local_managed_request(owner, None)
        .unwrap_or_else(|refusal| panic!("actual opted-in release: {:?}", refusal.cause()));
    assert_eq!(released.native().terminal().handle(), original);
    assert_eq!(
        released.native().terminal().status(),
        worth_signal::facade::ResourceInFlightStatus::Cancelled
    );
}
