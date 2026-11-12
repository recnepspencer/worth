//! Terminal retries expose the original source and never re-enter the handler.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolution as Resolution, WorthQueryReleasedApplicationEffect,
};

pub(super) fn observe(
    home: &Home,
    context: &JourneyContext,
    original: &OriginalCustody,
) -> WorthQueryReleasedApplicationEffect {
    let (_, live) = scope();
    let resolve = |value| {
        let principal = identity::inspection_authentication::authenticate(
            home.runtime().runtime().installed_schema(),
            &live,
        );
        home.runtime()
            .request(&principal, &live)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".into(),
                balance: value,
            })
            .without_source()
            .idempotency(&0x9921u64)
            .resolve_idempotency_in_program(home.runtime())
            .unwrap()
            .into_resolution()
    };
    let released = match resolve(9) {
        Resolution::ReleasedOriginal(released) => released,
        other => panic!("actual effect-bearing release keeps its original key: {other:?}"),
    };
    assert_eq!(
        serde_json::to_value(released.semantic_result()).unwrap(),
        original.native_meaning
    );
    assert_eq!(
        durable!(released.original_effect().identity()),
        durable!(&original.identity)
    );
    assert_eq!(
        durable!(released.original_effect().attempt_identity()),
        durable!(&original.attempt)
    );
    assert!(released.original_effect().original_association().is_some());
    assert!(released
        .original_effect()
        .release_journal_position()
        .is_some());
    assert!(released
        .original_effect()
        .release_artifact_digest()
        .is_some());
    assert!(matches!(resolve(10), Resolution::IntentDrift));
    assert_eq!(set(home.runtime(), &live, 0x9921, 9).unwrap_err().kind(),
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::OriginalEffectReleased);
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
    assert_eq!(balance(home.runtime(), &live), 8);
    released
}
