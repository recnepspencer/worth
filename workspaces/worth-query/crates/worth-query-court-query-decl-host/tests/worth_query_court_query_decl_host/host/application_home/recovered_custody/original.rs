//! The actual initial Host owner effect ends before any reopen frame exists.
use super::*;

pub(super) fn capture(
    context: &JourneyContext,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> OriginalCustody {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
    let request = fixture::request(
        context.location.clone(),
        context.build,
        context.initialized.clone(),
        context.contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(*execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual writer: {refused:?}"));
    let (_, live) = scope();
    assert!(matches!(
        set(home.runtime(), &live, 0x9920, 8).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
    ));
    home.runtime()
        .runtime()
        .world_operation_control_for_test()
        .panic_before_product_compare_once_for_test();
    let pending = pending(set(home.runtime(), &live, 0x9921, 9));
    let recovery = pending
        .try_into_recovery()
        .unwrap_or_else(|pending| panic!("same actual post-Native owner: {pending:?}"));
    let effects = recovery.inspect().unwrap();
    let original_identity = recovery.record_handle().identity().clone();
    let original_attempt = effects.attempt_identity().clone();
    let original_meaning = serde_json::to_value(effects.native_semantic_result().unwrap()).unwrap();
    let predecessor = effects.expected_product().selected_commit().clone();
    assert!(
        effects.owner_effect_count() > 0,
        "the actual control unwinds after the performed Native effect"
    );
    assert_eq!(balance(home.runtime(), &live), 8);
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);

    drop(effects);
    drop(recovery);
    drop(home);
    OriginalCustody {
        identity: original_identity,
        attempt: original_attempt,
        native_meaning: original_meaning,
        predecessor,
    }
}

pub(super) fn reopen(
    context: &JourneyContext,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Home {
    let request = fixture::request_with_initializer(
        context.location.clone(),
        context.build,
        context.contacts.clone(),
        |_, _, _| panic!("existing Home cannot initialize"),
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| panic!("typed original custody must become service: {refused:?}"))
}
