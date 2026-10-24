//! Native outcome unwinding preserves the provider claim of a combined source.
use super::super::{
    authentication::*, composite_definition_court as composite,
    composite_definition_handler::handler_contacts, composite_definition_model::Change, host,
    program::Program, query::Row, required_output_courts::head,
};
use std::{
    num::NonZeroUsize,
    panic::{catch_unwind, AssertUnwindSafe},
};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::*,
    runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause},
};
use worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest;

pub fn run() {
    let mut application = host::install();
    exercise(&application);
    application.close_conditional_runtime().unwrap();
    println!(
        "combined source recovery preserves its actual native successor through Signal unwind"
    );
}

fn exercise(application: &host::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let key = "combined-source-unwind".to_owned();
    let original_world = head(application);
    application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(application.runtime().product_runtime())
                .unwrap();
            application
                .runtime()
                .fail_next_durable_append_for_test(&execution);
        })
        .unwrap();
    let outcome = request
        .advanced()
        .mutate(intent())
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::ProductUnpublished(partial),
    ) = outcome
    else {
        panic!("real failed durable append retains the ordinary combined source: {outcome:?}");
    };
    assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
    assert_eq!(partial.owner_effect_count(), 1);
    let original = partial.into_recovery();
    let prepared_contacts = handler_contacts();
    assert_eq!(head(application), original_world);
    original.continue_owner_settlement().unwrap();
    let inspection = original.inspect().unwrap();
    let refused = request
        .advanced()
        .mutate(intent())
        .without_source()
        .idempotency(&key)
        .recover_unpublished_in_program::<Program>(&original, application);
    match refused {
        Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
            WorthQueryManagedApplicationRecoveryDenial::Adoption(
                RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
                    RuntimeWorldRecoveryDenial::CallerCapabilityLive,
                ),
            ),
        )) => {}
        Err(other) => panic!("the actual held inspector must refuse native preparation: {other:?}"),
        Ok(_) => panic!("the live original inspector cannot permit native recovery"),
    }
    drop(inspection);
    assert_eq!(head(application), original_world);
    assert_eq!(handler_contacts(), prepared_contacts);
    assert_eq!(original.inspect().unwrap().owner_effect_count(), 1);
    application
        .runtime()
        .signal_operation_control_for_test()
        .unwrap()
        .inject_panic_once_for_test(SignalOwnerOperationBoundaryForTest::OutcomeConstruction);
    let stopped = catch_unwind(AssertUnwindSafe(|| {
        request
            .advanced()
            .mutate(intent())
            .without_source()
            .idempotency(&key)
            .recover_unpublished_in_program::<Program>(&original, application)
    }));
    let panic = match stopped {
        Err(panic) => panic,
        Ok(Err(denial)) => panic!("recovery must reach the actual Signal fault: {denial:?}"),
        Ok(Ok(_)) => panic!("the actual Signal outcome fault must propagate"),
    };
    assert_eq!(
        panic.downcast_ref::<String>().map(String::as_str),
        Some("injected Signal owner operation fault at OutcomeConstruction")
    );
    assert_eq!(
        head(application),
        original_world,
        "neither component alone publishes the product"
    );
    assert_eq!(handler_contacts(), prepared_contacts);

    let page = application
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(2).unwrap())
        .unwrap();
    assert_eq!(
        page.rows().len(),
        2,
        "the predecessor and actual native successor retain their exact cleanup"
    );
    let successor = page
        .rows()
        .iter()
        .find(|row| row.handle() != original.record_handle())
        .expect("World retained its distinct pre-reserved successor");
    let recovery = application
        .runtime()
        .readmit_product_publication_recovery(successor.handle())
        .unwrap();
    {
        let effects = recovery.inspect().unwrap();
        assert_eq!(
            effects.owner_effect_count(),
            2,
            "the original Relational and actual moved Signal results survive"
        );
        assert!(!effects.relational_requires_settlement());
    }
    drop(page);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(application.installed_schema(), &fresh_scope);
    let fresh = application
        .runtime()
        .request(&fresh_principal, &fresh_scope);
    let outcome = fresh.advanced().mutate(intent()).without_source().idempotency(&key)
        .recover_unpublished_in_program::<Program>(&recovery, application)
        .unwrap_or_else(|denial| panic!("the retained native successor must keep its original Query provider claim: {denial:?}"));
    let WorthQueryManagedApplicationRecoveryOutcome::Performed(performed) = outcome else {
        panic!("original combined effects recover into one product publication");
    };
    let (read, publication, prior_cleanup, pending_cleanup) = performed.into_parts();
    assert!(publication.is_none());
    assert!(prior_cleanup.is_none());
    assert!(pending_cleanup.is_none());
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) =
        read.unwrap().into_resolution()
    else {
        panic!("the original key resolves its recovered receipt");
    };
    composite::assert_combined(&receipt);
    assert_eq!(
        handler_contacts(),
        prepared_contacts,
        "neither recovery reexecutes the handler"
    );
    assert_ne!(head(application), original_world);
    assert_eq!(
        host::read(application),
        [Row {
            primary: 23,
            secondary: 7
        }]
    );
    application
        .with_application_advancement(&fresh_scope, |phase| {
            application.release_product_publication_recovery_in_advancement(&phase, original, 0)
        })
        .unwrap()
        .unwrap();
    assert!(application
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
}

fn intent() -> Change {
    Change {
        key: "numeric".to_owned(),
        value: 23,
    }
}
