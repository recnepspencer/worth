//! An accepted Stable interest refreshes from a newly disclosed source.

use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryOutputSettlementPosture,
};

#[test]
fn open_stable_interest_refreshes_twice_without_new_producer_or_world_commit() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let performed = input_cutoff::direct_demand(&request, &application, "anchor-a");
    assert_eq!(
        performed.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    let original_receipt = performed.application_commit_receipt().unwrap().clone();

    edit_successor(&request, &application, "anchor-b", 2, 101);
    let mut interest = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("the changed source admits an output interest");
    let first = settle_open(&mut interest, &request);
    assert_eq!(
        first.posture(),
        WorthQueryOutputSettlementPosture::StableReused
    );
    assert_eq!(first.producer_contacts_in_this_demand(), 0);
    assert!(first.application_commit_receipt().is_none());

    edit_successor(&request, &application, "anchor-b", 3, 102);
    let before = request.retain_read().unwrap();
    let second = settle_open(&mut interest, &request);
    let after = request.retain_read().unwrap();
    assert_eq!(
        second.posture(),
        WorthQueryOutputSettlementPosture::StableReused
    );
    assert_eq!(second.producer_contacts_in_this_demand(), 0);
    assert!(second.application_commit_receipt().is_none());
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(
        original_receipt
            .output_correspondence()
            .entity(
                worth_query_host::facade::primary_graph::WorthQueryApplicationOutputRole::<
                    PlanarMutationBinding<CheckpointSchema>,
                    Body,
                    worth_query_host::facade::primary_graph::WorthQueryPreserveOutput,
                >::from_static("anchor")
            )
            .unwrap()
            .entity_id(),
        second
            .output_correspondence()
            .entity(
                worth_query_host::facade::primary_graph::WorthQueryApplicationOutputRole::<
                    PlanarMutationBinding<CheckpointSchema>,
                    Body,
                    worth_query_host::facade::primary_graph::WorthQueryPreserveOutput,
                >::from_static("anchor")
            )
            .unwrap()
            .entity_id(),
    );

    edit_successor(&request, &application, "anchor-b", 4, 103);
    let before = request.retain_read().unwrap();
    let third = settle_open(&mut interest, &request);
    let after = request.retain_read().unwrap();
    assert_eq!(
        third.posture(),
        WorthQueryOutputSettlementPosture::StableReused
    );
    assert_eq!(third.producer_contacts_in_this_demand(), 0);
    assert!(third.application_commit_receipt().is_none());
    assert_eq!(before.selected_commit(), after.selected_commit());
}

fn controls() -> WorthQueryOutputDemandControls {
    input_cutoff::controls()
}

fn settle_open<'application>(
    interest: &mut worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandHandle<
        'application,
        CheckpointSchema,
        PlanarOutputDemand,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        '_,
        '_,
        CheckpointSchema,
    >,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
    PlanarQuery,
> {
    for _ in 0..256 {
        match interest.advance(request).expect("the open demand advances") {
            WorthQueryApplicationOutputDemandProgress::Pending => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => return settled,
        }
    }
    panic!("the open demand settles within its bounded progression")
}

fn edit_successor(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
    application: &support::Application,
    successor: &str,
    replacement_y: u64,
    idempotency: u64,
) {
    let selected = request
        .query(PlanarRead {
            body_key: successor.to_owned(),
        })
        .execute()
        .expect("the successor is selected");
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: successor.to_owned(),
            replacement_y: length(replacement_y),
        })
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&idempotency)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(application)
        .expect("the successor edit publishes");
}
