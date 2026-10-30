pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_safe_retry_pending::*;

#[test]
fn safe_retry_names_a_pending_completion_publication_then_the_completion() {
    let (world, handle, token) =
        completed_before_the_source_installs("safe-retry-publication-pending", 83);
    let handle = refused_retry(
        &world,
        handle,
        BankRecoveryDenialKind::CompletionPublicationPending,
    );
    let route = install_source(&world);
    refused_retry(&world, handle, BankRecoveryDenialKind::AlreadyCompleted);
    assert!(
        world
            .fixture
            .world
            .runtime
            .observe_estate_rail_completion(&route, token)
            .is_some(),
        "the retry published the retained completion into the terminal owner"
    );
}
