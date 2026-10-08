//! Public Clean reuse skips the actual Query read kernel; a changed source enters it.

use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationOutputDemandProgress, WorthQueryOutputSettlementPosture,
    },
    primary_graph::query_read_kernel_entries_on_this_thread_for_test as query_entries,
};

#[test]
fn clean_ready_advancement_skips_source_query_but_changed_input_reenters_it() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("the real output interest starts under the installed host profile");
    let first = settle_open(&mut interest, &request);
    assert_eq!(
        first.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(first.producer_contacts_in_this_demand(), 1);
    let original_entity = first
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();

    for _ in 0..2 {
        let before_basis = request.retain_read().unwrap();
        let before_entries = query_entries();
        let settled = match interest
            .advance(&request)
            .expect("the clean interest advances")
        {
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
            WorthQueryApplicationOutputDemandProgress::Pending => {
                panic!("Clean Ready must settle in this advance")
            }
        };
        assert_eq!(
            query_entries(),
            before_entries,
            "Clean advancement must not enter the source Query kernel"
        );
        assert_eq!(settled.producer_contacts_in_this_demand(), 0);
        assert_eq!(
            settled
                .outputs_of::<PlanarOutputs>()
                .unwrap()
                .entity::<PlanarAnchorOutput<CheckpointSchema>>()
                .unwrap()
                .entity_id(),
            original_entity
        );
        let after_basis = request.retain_read().unwrap();
        assert_eq!(
            before_basis.selected_commit(),
            after_basis.selected_commit()
        );
    }

    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&111_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("a real changed producer input publishes");
    drop(source);
    let before_changed_entries = query_entries();
    let changed = settle_open(&mut interest, &request);
    assert!(
        query_entries() > before_changed_entries,
        "the dirty source must enter the actual Query kernel"
    );
    assert_eq!(
        changed.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(changed.producer_contacts_in_this_demand(), 1);
    assert_eq!(
        changed
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        original_entity
    );
}

fn settle_open<'application>(
    interest: &mut worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandHandle<
        'application, CheckpointSchema, PlanarOutputDemand,
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
        match interest
            .advance(request)
            .expect("the real interest advances")
        {
            WorthQueryApplicationOutputDemandProgress::Pending => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => return settled,
        }
    }
    panic!("the interest did not settle within its bounded progression")
}
