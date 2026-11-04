//! A fresh final Host boundary proves the chosen disposition stayed durable.
use super::*;

pub(super) fn verify(
    context: &JourneyContext,
    original: &OriginalCustody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    continue_original: bool,
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
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
    assert_eq!(context.initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("readmitted disposition close: {refused:?}"));
}
