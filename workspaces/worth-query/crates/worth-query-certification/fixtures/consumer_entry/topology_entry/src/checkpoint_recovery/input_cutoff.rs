//! Contact-free input cutoff through installed production owners and public APIs.

use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationOutputDemandSettlement,
    WorthQueryOutputSettlementPosture,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOutputRole, WorthQueryPreserveOutput,
};

#[test]
fn fresh_source_suffix_with_equal_declared_input_reuses_without_a_handler_or_world_commit() {
    exercise("anchor-a", "anchor-b", 2, true, controls());
}

#[test]
fn raw_native_reader_prevents_input_cutoff_even_when_the_declared_input_is_equal() {
    exercise("anchor-isolated", "anchor-island", 51, false, controls());
}

#[test]
fn undeclared_key_consumption_prevents_input_cutoff_even_when_the_declared_input_is_equal() {
    exercise("anchor-island", "anchor-atoll", 61, false, controls());
}

#[test]
fn bounded_currentness_work_preserves_performed_provenance_and_selects_fresh_execution() {
    exercise(
        "anchor-a",
        "anchor-b",
        2,
        false,
        controls().source_currentness_work(NonZeroUsize::new(8_192).unwrap()),
    );
}

fn exercise(
    root: &str,
    changed_successor: &str,
    replacement_y: u64,
    expect_reuse: bool,
    demand_controls: WorthQueryOutputDemandControls,
) {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let first =
        direct_demand_with_controls(&request, &application, root, demand_controls, "initial");
    assert_eq!(first.producer_contacts_in_this_demand(), 1);
    let original_receipt = first.application_commit_receipt().unwrap().clone();
    let original_entity = first
        .output_correspondence()
        .entity(WorthQueryApplicationOutputRole::<
            PlanarMutationBinding<CheckpointSchema>,
            Body,
            WorthQueryPreserveOutput,
        >::from_static("anchor"))
        .unwrap()
        .entity_id();
    let before_input = request
        .query(PlanarRead {
            body_key: root.to_owned(),
        })
        .execute()
        .unwrap();
    let changed_source = request
        .query(PlanarRead {
            body_key: changed_successor.to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: changed_successor.to_owned(),
            replacement_y: length(replacement_y),
        })
        .expect_source(changed_source.observed_sources()[0].clone())
        .idempotency(&91_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("the independently selected successor edit publishes");
    let edited = request
        .query(PlanarRead {
            body_key: changed_successor.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        edited.rows()[0].y,
        length(replacement_y),
        "the independent native source edit is visible"
    );
    drop(edited);
    let after_input = request
        .query(PlanarRead {
            body_key: root.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        before_input.rows(),
        after_input.rows(),
        "source dependency changes without changing the producer input value"
    );
    drop((before_input, after_input, changed_source));
    let before = request.retain_read().unwrap();
    let settled = direct_demand_with_controls(
        &request,
        &application,
        root,
        demand_controls,
        "edited source",
    );
    let after = request.retain_read().unwrap();
    if expect_reuse {
        assert_eq!(
            settled.posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(settled.producer_contacts_in_this_demand(), 0);
        assert!(settled.application_commit_receipt().is_none());
        assert_eq!(before.selected_commit(), after.selected_commit());
        let evidence = settled
            .readiness_delivery()
            .expect("stable publication exposes zero execution work");
        assert_eq!(evidence.producer_contact_count(), 0);
        assert_eq!(evidence.delivery_contact_count(), 0);
        assert_eq!(evidence.signal_seeds_emitted(), 0);
        assert_eq!(evidence.semantic_observation_reads(), 0);
        assert_eq!(
            settled
                .output_correspondence()
                .entity(WorthQueryApplicationOutputRole::<
                    PlanarMutationBinding<CheckpointSchema>,
                    Body,
                    WorthQueryPreserveOutput,
                >::from_static("anchor"),)
                .unwrap()
                .entity_id(),
            original_entity
        );
        assert_eq!(
            original_receipt
                .output_correspondence()
                .entity(WorthQueryApplicationOutputRole::<
                    PlanarMutationBinding<CheckpointSchema>,
                    Body,
                    WorthQueryPreserveOutput,
                >::from_static("anchor"),)
                .unwrap()
                .entity_id(),
            original_entity
        );
        drop((before, after));
        let successor = request
            .query(PlanarRead {
                body_key: changed_successor.to_owned(),
            })
            .execute()
            .unwrap();
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: changed_successor.to_owned(),
                replacement_y: length(replacement_y + 1),
            })
            .expect_source(successor.observed_sources()[0].clone())
            .idempotency(&92_u64)
            .execute_performed::<CheckpointProgram, CheckpointRoot>(&application)
            .expect("the successor edit invalidates the first stable alias");
        drop(successor);
        let second_basis = request.retain_read().unwrap();
        let repeated = direct_demand_with_controls(
            &request,
            &application,
            root,
            demand_controls,
            "repeated alias",
        );
        let repeated_basis = request.retain_read().unwrap();
        assert_eq!(
            repeated.posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(repeated.producer_contacts_in_this_demand(), 0);
        assert!(repeated.application_commit_receipt().is_none());
        assert_eq!(
            second_basis.selected_commit(),
            repeated_basis.selected_commit()
        );
        assert_eq!(
            repeated
                .output_correspondence()
                .entity(WorthQueryApplicationOutputRole::<
                    PlanarMutationBinding<CheckpointSchema>,
                    Body,
                    WorthQueryPreserveOutput,
                >::from_static("anchor"))
                .unwrap()
                .entity_id(),
            original_entity,
        );
    } else {
        assert_eq!(
            settled.posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        assert_eq!(settled.producer_contacts_in_this_demand(), 1);
        assert!(settled.application_commit_receipt().is_some());
    }
}

pub(super) fn direct_demand<'application, 'principal, 'scope>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >,
    application: &'application support::Application,
    root: &str,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    direct_demand_with_controls(request, application, root, controls(), "installed demand")
}

pub(super) fn controls() -> WorthQueryOutputDemandControls {
    let source_work =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
            .limits()
            .source_currentness_work();
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
    .source_currentness_work(NonZeroUsize::new(source_work).unwrap())
}

fn direct_demand_with_controls<'application, 'principal, 'scope>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >,
    application: &'application support::Application,
    root: &str,
    demand_controls: WorthQueryOutputDemandControls,
    stage: &str,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    let mut demand = request
        .demand(PlanarOutputDemand::new(root))
        .controls(demand_controls)
        .start_in_program::<CheckpointProgram, CheckpointRoot>(application)
        .unwrap_or_else(|denial| panic!("{stage} demand failed to start: {denial:?}"));
    for _ in 0..256 {
        match demand
            .advance(request)
            .unwrap_or_else(|denial| panic!("{stage} demand failed to advance: {denial:?}"))
        {
            WorthQueryApplicationOutputDemandProgress::Pending => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => return settled,
        }
    }
    panic!("the demand did not settle within its synchronous bound")
}
