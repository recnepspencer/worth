//! Installed typed quote overflow precedes effects; correcting the source settles.
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryApplicationEntitySeed,
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn checked_source_quote_overflow_is_not_foreign_or_a_policy_refusal_and_repair_settles() {
    let _guard = checkpoint_recovery_test_guard();
    {
        let application = installation::install_ordinary(seed);
        exercise(&application, None);
    }
    {
        let application = installation::install_program(seed);
        exercise(application.runtime(), Some(&application));
    }
}

fn exercise(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<QuoteSchema>,
    program: Option<
        &application_installation::WorthQueryProgramApplicationRuntime<
            QuoteSchema,
            program::Program,
        >,
    >,
) {
    let (scope, principal) = authenticate(application);
    let request = application.request(&principal, &scope);
    operation::OPERATIONS.store(0, Ordering::SeqCst);
    let before = request
        .query(Read {
            key: "quote-source".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(before.rows()[0].count, u64::MAX);
    drop(before);
    let before_basis = request.retain_current_read().unwrap();
    let commit = before_basis.selected_commit().clone();
    drop(before_basis);
    let demand = request.demand(producer::Demand("quote-source".into()));
    let started = match program {
        Some(application) => {
            demand.start_in_program::<program::Program, program::Root>(application)
        }
        None => demand.start(),
    };
    let refusal = match started {
        Ok(_) => panic!("checked retained vector extent must overflow before allocation"),
        Err(refusal) => refusal,
    };
    let WorthQueryApplicationOutputDemandDenial::Demand(cause) = refusal else {
        panic!("the installed quote owns the refusal")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOutputDemandDenialKind::ProducerResourceQuote(
            WorthQueryProducerResourceQuoteDenial::SizeOverflow
        )
    );
    assert_eq!(cause.subject(), "quote-count.producer");
    assert_eq!(operation::OPERATIONS.load(Ordering::SeqCst), 0);
    let after = request
        .query(Read {
            key: "quote-source".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(after.rows()[0].count, u64::MAX);
    drop(after);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &commit
    );
    let correction = request
        .advanced()
        .mutate(Edit {
            scope_key: "quote-source".into(),
            count: 2,
        })
        .idempotency(&0x51554f54u64);
    let corrected = match program {
        Some(application) => correction.execute_in_program::<program::Program>(
            application,
            ExecutionAllocationPolicy::SystemAllocation,
        ),
        None => correction.execute(ExecutionAllocationPolicy::SystemAllocation),
    }
    .unwrap();
    assert!(
        matches!(
            corrected,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "source correction must commit: {corrected:?}"
    );
    let demand = request.demand(producer::Demand("quote-source".into()));
    let mut ordinary = match program {
        Some(application) => {
            demand.start_in_program::<program::Program, program::Root>(application)
        }
        None => demand.start(),
    }
    .expect("finite original source quote is admitted");
    let settled = (0..64)
        .find_map(|_| match ordinary.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
            WorthQueryApplicationOutputDemandProgress::Settled(s) => Some(s),
        })
        .expect("Fresh producer settles after source correction");
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let actual = request
        .query(output::OutputRead {
            key: "quote-source".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(actual.rows()[0].total, 2);
    assert_eq!(actual.rows()[0].key, "quote-source");
    drop(actual);
    drop(settled);
    drop(ordinary);
    assert_eq!(operation::OPERATIONS.load(Ordering::SeqCst), 1);
    let edit = request
        .advanced()
        .mutate(Edit {
            scope_key: "quote-source".into(),
            count: 3,
        })
        .idempotency(&0x51554f55u64);
    let edited = match program {
        Some(application) => edit.execute_in_program::<program::Program>(
            application,
            ExecutionAllocationPolicy::SystemAllocation,
        ),
        None => edit.execute(ExecutionAllocationPolicy::SystemAllocation),
    }
    .unwrap();
    assert!(
        matches!(
            edited,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "upstream edit must commit: {edited:?}"
    );
    let demand = request.demand(producer::Demand("quote-source".into()));
    let mut selected = match program {
        Some(application) => {
            demand.start_in_program::<program::Program, program::Root>(application)
        }
        None => demand.start(),
    }
    .unwrap();
    let settled = (0..64)
        .find_map(|_| match selected.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
            WorthQueryApplicationOutputDemandProgress::Settled(s) => Some(s),
        })
        .expect("Fresh regeneration keeps the same fallible quote boundary");
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert_eq!(operation::OPERATIONS.load(Ordering::SeqCst), 2);
    let actual = request
        .query(output::OutputRead {
            key: "quote-source".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(actual.rows()[0].total, 3);
    assert_eq!(actual.rows()[0].key, "quote-source");
    drop(actual);
    drop(settled);
    drop(selected);
}
fn seed(graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<QuoteSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Body::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("quote-source").unwrap(),
            )
            .field(BodyKey::reference(), "quote-source".to_owned())
            .field(RequestedRows::reference(), u64::MAX)
            .field(CountedTotal::reference(), 0),
        )
        .unwrap();
}
