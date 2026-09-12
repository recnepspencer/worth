//! Run with `cargo run -p worth-query-certification --example ordinary_product_workflow`.

pub mod product_workflow_support;

use std::sync::Arc;

use product_workflow_support::schema::{IntentIdentityField, TemporalIntentQuery};
use product_workflow_support::{
    controls, principal, product_identity, CompletingExternalTransport, ExampleApplication,
    ReplacementPredicate,
};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph, product, runtime,
};

fn main() {
    std::thread::Builder::new()
        .name("ordinary-product-workflow".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("the bounded example thread must start")
        .join()
        .expect("the ordinary product workflow must not unwind")
        .unwrap_or_else(|stop| eprintln!("application stopped with retained evidence: {stop:?}"));
}

fn run() -> Result<(), NonCommitted> {
    let mut application = ExampleApplication::publish("blocked");
    let transport = Arc::new(CompletingExternalTransport::default());
    application
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("the application transport must install once");

    let request = product_workflow_support::adapters::request_scope();
    let principal = principal(&application, &request);
    let branch = application.runtime.current_world();
    let selected = application
        .runtime
        .on_branch(branch)
        .select()
        .expect("the current product must be selectable");
    let original_commit = selected.product().selected_commit().clone();
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the selected product must resolve the requested entity");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let query = application
        .runtime
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .expect("the application query must be installed");
    let retained_read = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .expect("the exact selected product must admit the read");

    let admitted_change = application.admit_combined_change(
        application
            .runtime
            .on_branch(branch)
            .select()
            .expect("the mutation product must be selectable"),
        "published-through-world",
        Arc::new(ReplacementPredicate),
    );
    let outcome = application
        .runtime
        .on_branch(branch)
        .transaction()
        .apply(admitted_change)
        .commit()
        .expect("the admitted change and transaction must name the same product");
    let mut committed = require_committed(&application.runtime, outcome)?;
    assert_eq!(committed.product_branch(), branch);
    assert_eq!(
        committed
            .committed_product_publication()
            .relational_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        committed.committed_product_publication().signal_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(transport.contact_count(), 1);

    let change = committed
        .take_performed_relational_product_change()
        .expect("a committed Relational change must carry its delivery patch");
    let current = application
        .runtime
        .on_branch(branch)
        .select()
        .expect("the published product occurrence must be selectable");
    let delivery = current
        .deliver_relational_change_to_conditional(&application.clock, 0, change)
        .expect("the exact product must admit its patch");
    let runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) =
        delivery
    else {
        panic!("the ordinary patch must be delivered")
    };
    assert_eq!(delivery.source_envelopes_loaded(), 1);
    drop(delivery);
    drop(current);

    application.clock_control.push(3, 11);
    let conditional = application
        .runtime
        .on_branch(branch)
        .select()
        .expect("the published product must remain selectable")
        .conditional_clock(&application.clock)
        .expect("the conditional belongs to this product")
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(conditional) =
        conditional
    else {
        panic!("the delivered patch must execute its conditional")
    };
    assert_eq!(conditional.committed_operation_count(), 1);
    drop(conditional);

    let retained = application
        .runtime
        .execute_application_query_one_shot(retained_read)
        .expect("the retained read must remain executable");
    assert_eq!(retained.rows()[0].input, "payload");
    assert_eq!(
        product_identity(retained.receipt()).selected_commit(),
        &original_commit
    );
    drop(retained);
    drop(committed);

    application
        .runtime
        .close_conditional_runtime()
        .expect("the application runtime must cleanly close");
    println!("published, delivered, executed, retained, and cleaned up product {branch:?}");
    Ok(())
}

enum NonCommitted {
    ProductStale(primary_graph::WorthQueryProductStaleApplication),
    ProductUnpublishedRecovered {
        cause: String,
        owner_effect_count: usize,
        settlement_continued: bool,
        recovery_slots_examined: usize,
    },
    ProductUnpublishedRecoveryPending(
        primary_graph::WorthQueryProductUnpublishedRecoveryReleaseFailure,
    ),
    NoEffect(product::WorthQueryApplicationNoEffectCause),
    Stale(primary_graph::WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(primary_graph::WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(primary_graph::WorthQueryApplicationCommitDeferred),
    SettlementRecovered {
        next_action: primary_graph::WorthQueryApplicationSettlementNextAction,
        recovered_commit: String,
    },
    SettlementRecoveryPending {
        deferred: primary_graph::WorthQueryApplicationSettlementDeferred,
        error: primary_graph::WorthQueryApplicationSettlementRecoveryError,
    },
    Indeterminate(primary_graph::WorthQueryApplicationUnresolvedCommitEvidence),
}

impl std::fmt::Debug for NonCommitted {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProductStale(stale) => formatter
                .debug_struct("ProductStale")
                .field("expected", stale.expected_product().selected_commit())
                .field("observed", &stale.observed_product())
                .finish(),
            Self::ProductUnpublishedRecovered {
                cause,
                owner_effect_count,
                settlement_continued,
                recovery_slots_examined,
            } => formatter
                .debug_struct("ProductUnpublishedRecovered")
                .field("cause", cause)
                .field("owner_effect_count", owner_effect_count)
                .field("settlement_continued", settlement_continued)
                .field("recovery_slots_examined", recovery_slots_examined)
                .finish(),
            Self::ProductUnpublishedRecoveryPending(failure) => formatter
                .debug_tuple("ProductUnpublishedRecoveryPending")
                .field(failure)
                .finish(),
            Self::NoEffect(cause) => formatter.debug_tuple("NoEffect").field(cause).finish(),
            Self::Stale(stale) => formatter
                .debug_struct("Stale")
                .field("stale_fact_count", &stale.stale_fact_count())
                .finish(),
            Self::Cancelled => formatter.write_str("Cancelled"),
            Self::TimedOut => formatter.write_str("TimedOut"),
            Self::Denied(denial) => formatter
                .debug_struct("Denied")
                .field("kind", &denial.kind())
                .field("stage", &denial.stage())
                .finish(),
            Self::Aborted => formatter.write_str("Aborted"),
            Self::Deferred(deferred) => formatter
                .debug_struct("Deferred")
                .field("kind", &deferred.kind())
                .field("stage", &deferred.stage())
                .field("detail", &deferred.detail())
                .finish(),
            Self::SettlementRecovered {
                next_action,
                recovered_commit,
            } => formatter
                .debug_struct("SettlementRecovered")
                .field("next_action", next_action)
                .field("recovered_commit", recovered_commit)
                .finish(),
            Self::SettlementRecoveryPending { deferred, error } => formatter
                .debug_struct("SettlementRecoveryPending")
                .field("next_action", &deferred.next_action())
                .field("stage", &deferred.stage())
                .field("error", error)
                .finish(),
            Self::Indeterminate(evidence) => formatter
                .debug_struct("Indeterminate")
                .field("recovery", &evidence.recovery())
                .field("stage", &evidence.stage())
                .field("detail", &evidence.detail())
                .finish(),
        }
    }
}

fn require_committed(
    runtime: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        product_workflow_support::schema::TemporalHostSchema,
    >,
    outcome: primary_graph::WorthQueryApplicationCommitOutcome,
) -> Result<primary_graph::WorthQueryApplicationCommitReceipt, NonCommitted> {
    match outcome {
        primary_graph::WorthQueryApplicationCommitOutcome::Committed(receipt)
        | primary_graph::WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
            Ok(receipt)
        }
        primary_graph::WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            Err(NonCommitted::ProductStale(stale))
        }
        primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) => {
            let cause = format!("{:?}", partial.cause());
            let owner_effect_count = partial.owner_effect_count();
            let settlement_owed = partial.relational_requires_settlement();
            let recovery_slots_examined = runtime
                .product_publication_recovery_page(None, std::num::NonZeroUsize::new(1).unwrap())
                .map(|page| page.examined())
                .unwrap_or(0);
            let recovery = partial.into_recovery();
            let settlement_continued =
                !settlement_owed || recovery.continue_owner_settlement().is_ok();
            match runtime.release_product_publication_recovery(recovery, 0) {
                Ok(_) => Err(NonCommitted::ProductUnpublishedRecovered {
                    cause,
                    owner_effect_count,
                    settlement_continued,
                    recovery_slots_examined,
                }),
                Err(failure) => Err(NonCommitted::ProductUnpublishedRecoveryPending(failure)),
            }
        }
        primary_graph::WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => {
            Err(NonCommitted::NoEffect(no_effect.cause()))
        }
        primary_graph::WorthQueryApplicationCommitOutcome::Stale(stale) => {
            Err(NonCommitted::Stale(stale))
        }
        primary_graph::WorthQueryApplicationCommitOutcome::Cancelled => {
            Err(NonCommitted::Cancelled)
        }
        primary_graph::WorthQueryApplicationCommitOutcome::TimedOut => Err(NonCommitted::TimedOut),
        primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial) => {
            Err(NonCommitted::Denied(denial))
        }
        primary_graph::WorthQueryApplicationCommitOutcome::Aborted => Err(NonCommitted::Aborted),
        primary_graph::WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            Err(NonCommitted::Deferred(deferred))
        }
        primary_graph::WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            let next_action = deferred.next_action();
            match runtime.recover_deferred_application_settlement(&deferred) {
                Ok(commit) => Err(NonCommitted::SettlementRecovered {
                    next_action,
                    recovered_commit: format!("{commit:?}"),
                }),
                Err(error) => Err(NonCommitted::SettlementRecoveryPending { deferred, error }),
            }
        }
        primary_graph::WorthQueryApplicationCommitOutcome::Indeterminate(evidence) => {
            Err(NonCommitted::Indeterminate(evidence))
        }
    }
}
