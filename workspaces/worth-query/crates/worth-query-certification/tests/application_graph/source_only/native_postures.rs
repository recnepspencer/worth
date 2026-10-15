//! Actual pre-World native candidate races preserve terminal source custody.
use super::{fixture::*, readback::*, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    application_entry::*, declaration::application_schema::*, primary_graph::*,
};

#[test]
fn source_only_product_stale_terminal_preserves_original_meaning_and_close() {
    let host = install();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original_head = request.retain_current_read().unwrap();
    let record = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .unwrap();
    let original = request
        .mutate(OriginalRetention {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 19,
            },
        })
        .without_source()
        .preconditions(
            TypedMutationPreconditions::new()
                .expect_fact(
                    DocumentIdentityField::reference(),
                    ApplicationEncodedScalarValue::try_new(DOCUMENT_IDENTITY.to_owned()).unwrap(),
                )
                .expect_fact(
                    DocumentRetentionField::reference(),
                    ApplicationEncodedScalarValue::try_new(7_u64).unwrap(),
                ),
        )
        .idempotency_owned(0x9176_6019)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<SourceOnlyRetentionProgram, RetentionAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("original descriptor: {cause:?}"));
    drop(request);
    let pause = host
        .runtime()
        .pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
    let terminal = std::thread::scope(|threads| {
        let advancing = threads.spawn(|| {
            let scope = request_scope();
            let principal = authenticate_operator(host.installed_schema(), &scope);
            original.advance(&host.runtime().request(&principal, &scope))
        });
        assert!(pause.wait_until_reached(Duration::from_secs(10)));
        assert_eq!(host.contacts.counts(), (1, 1));
        host.runtime()
            .publish_native_field_write_for_test(
                branch,
                record,
                DocumentRetentionField::reference(),
                21_u64,
                &scope,
            )
            .unwrap();
        let winner = observe_head(host.runtime(), branch);
        pause.release();
        let terminal = match advancing.join().unwrap() {
            WorthQuerySourceOnlyAdvance::Terminal(owner) => owner,
            _ => panic!("native product-head loss is terminal, never retryable"),
        };
        assert!(
            matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::ProductStale(stale)))
            if stale.expected_product().selected_commit() == original_head.selected_commit()
                && stale.observed_product().is_none())
        );
        assert_eq!(
            observe_head(host.runtime(), branch).selected_commit(),
            winner.selected_commit()
        );
        assert_eq!(read_retention(host.runtime(), branch), 21);
        terminal
    });
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    let request = host.runtime().request(&principal, &stopped);
    let (terminal, cause) = terminal
        .close(&request)
        .err()
        .expect("close cannot bypass a stopped scope");
    assert!(matches!(
        cause,
        WorthQueryOperationCloseDenial::Execution(_)
    ));
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::ProductStale(_)
        ))
    ));
    drop(request);
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    terminal
        .close(&host.runtime().request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("fresh terminal close: {cause:?}"));
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(read_retention(host.runtime(), branch), 21);
}

#[test]
fn source_only_unchanged_native_candidate_completes_once() {
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let completed = match operation(&host, &request, 19, 0x9176_6119).advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(done) => done,
        _ => panic!("uncontended actual candidate commits"),
    };
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    completed
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("valid close: {cause:?}"));
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
}
