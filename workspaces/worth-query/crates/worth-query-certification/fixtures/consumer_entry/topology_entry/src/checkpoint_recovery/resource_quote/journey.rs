//! Installed producers publish and regenerate from their semantic source count.
use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryApplicationEntitySeed,
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn ordinary_and_selected_producers_publish_and_regenerate_without_resource_estimates() {
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
    assert_eq!(before.rows()[0].count, 2);
    drop(before);
    let demand = request.demand(producer::Demand("quote-source".into()));
    let mut ordinary = match program {
        Some(application) => {
            demand.start_in_program::<program::Program, program::Root>(application)
        }
        None => demand.start(),
    }
    .expect("ordinary source demand starts without a resource estimate");
    let settled = (0..64)
        .find_map(|_| match ordinary.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
            WorthQueryApplicationOutputDemandProgress::Settled(s) => Some(s),
        })
        .expect("producer settles from its original source");
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
        .expect("producer regenerates after the semantic source edit");
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
            .field(RequestedRows::reference(), 2)
            .field(CountedTotal::reference(), 0),
        )
        .unwrap();
}
