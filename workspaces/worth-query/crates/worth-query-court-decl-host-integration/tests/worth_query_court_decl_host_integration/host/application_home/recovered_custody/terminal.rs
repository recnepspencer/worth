//! A fresh final Host boundary proves the chosen disposition stayed durable.
use super::*;

pub(super) fn verify(
    context: &JourneyContext,
    original: &OriginalCustody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    continue_original: bool,
    released: Option<&worth_query_host::facade::primary_graph::WorthQueryReleasedApplicationEffect>,
) {
    let original_meaning = original.native_meaning.clone();
    let (_, live) = scope();
    let home = super::original::reopen(context, execution);
    assert_eq!(
        balance(home.runtime(), &live),
        if continue_original { 9 } else { 8 }
    );
    assert!(home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(32).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    if continue_original {
        match set(home.runtime(), &live, 0x9921, 9).unwrap() {
            Mutation::RecoveredCommitted(receipt) => assert_eq!(
                serde_json::to_value(receipt.semantic_result().semantic()).unwrap(),
                original_meaning
            ),
            other => {
                panic!("continued occurrence must remain original and published once: {other:?}")
            }
        }
    }
    if let Some(released) = released {
        let recovered = super::released::observe(&home, context, original);
        // Checkpoint and reopen preserve every original binding and physical-source
        // field; World identities compare through their Home-scoped projection.
        same_released_original(&recovered, released);
    }
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
    assert_eq!(context.initialized.load(Ordering::SeqCst), 1);
    home.close(execution)
        .unwrap_or_else(|refused| panic!("readmitted disposition close: {refused:?}"));
}

fn same_released_original(
    recovered: &worth_query_host::facade::primary_graph::WorthQueryReleasedApplicationEffect,
    released: &worth_query_host::facade::primary_graph::WorthQueryReleasedApplicationEffect,
) {
    assert_eq!(
        recovered.idempotency_key_identity(),
        released.idempotency_key_identity()
    );
    assert_eq!(recovered.recorded_intent(), released.recorded_intent());
    assert_eq!(recovered.recorded_affinity(), released.recorded_affinity());
    assert_eq!(recovered.outcome_identity(), released.outcome_identity());
    assert_eq!(
        recovered.emitted_effect_count(),
        released.emitted_effect_count()
    );
    assert_eq!(recovered.semantic_result(), released.semantic_result());
    assert_eq!(
        recovered.exact_output_settlement(),
        released.exact_output_settlement()
    );
    let (recovered, released) = (recovered.original_effect(), released.original_effect());
    assert_eq!(
        durable!(recovered.identity()),
        durable!(released.identity())
    );
    assert_eq!(
        durable!(recovered.attempt_identity()),
        durable!(released.attempt_identity())
    );
    assert_eq!(
        recovered
            .reserved_commit_identity()
            .map(|commit| durable!(commit)),
        released
            .reserved_commit_identity()
            .map(|commit| durable!(commit))
    );
    assert_eq!(recovered.expected_product(), released.expected_product());
    assert_eq!(
        recovered.original_association(),
        released.original_association()
    );
    assert_eq!(
        recovered.release_journal_position(),
        released.release_journal_position()
    );
    assert_eq!(
        recovered.release_artifact_digest(),
        released.release_artifact_digest()
    );
    assert_eq!(recovered.native_position(), released.native_position());
    assert_eq!(
        recovered.native_artifact_digest(),
        released.native_artifact_digest()
    );
    assert_eq!(
        recovered.native_semantic_result(),
        released.native_semantic_result()
    );
}
