//! A held required artifact follows its family's actual Preserve binding.
use super::*;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn one_binding_with_both_postures_keeps_a_distinct_preserve_row() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_no_reuse(None);
    let (scope, principal) = support::authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut initial = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<NoReuseProgram, program::Root>(&application)
        .unwrap();
    let first = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the dual-posture binding initially settles");
    let entity = first
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<NoReuseSchema>>()
        .unwrap()
        .entity_id();
    drop(first);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let selected = application
        .select_output_producer::<PlanarOutputFamily>(&source.observed_sources()[0], "planar", 4096)
        .unwrap();
    assert_eq!(
        selected.applicability().lifecycle(),
        WorthQueryProducerLifecyclePosture::Preserve
    );
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: support::length(9),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_6102_u64)
        .execute_performed::<NoReuseProgram, program::Root>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let second = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the same binding advances its Preserve row");
    assert_eq!(
        second
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<NoReuseSchema>>()
            .unwrap()
            .entity_id(),
        entity
    );
}

#[test]
fn held_required_initial_refreshes_through_preserve_and_reopens_current() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_no_reuse(None);
    let (scope, principal) = support::authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut initial = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(&application)
        .unwrap();
    let first = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the real Initial output settles");
    let entity = first
        .outputs_of::<FinalPlanarOutputs>()
        .unwrap()
        .entity::<FinalAnchorOutput<NoReuseSchema>>()
        .unwrap()
        .entity_id();
    drop(first);

    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: support::length(9),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_6101_u64)
        .execute_performed::<NoReuseProgram, program::Root>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let second = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the held Initial demand follows the actual Preserve successor");
    assert_eq!(
        second
            .outputs_of::<FinalPlanarPreserveOutputs>()
            .unwrap()
            .entity::<FinalPreservedAnchorOutput<NoReuseSchema>>()
            .unwrap()
            .entity_id(),
        entity
    );
    drop(second);
    assert!(matches!(
        initial.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    drop(initial);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(application);
    let reopened = install_no_reuse(Some(checkpoint));
    let (scope, principal) = support::authenticate(&reopened);
    let request = reopened.request(&principal, &scope);
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(&reopened)
        .unwrap();
    let restored = (0..64)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the current checkpoint reopens");
    assert_eq!(restored.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        restored
            .outputs_of::<FinalPlanarPreserveOutputs>()
            .unwrap()
            .entity::<FinalPreservedAnchorOutput<NoReuseSchema>>()
            .unwrap()
            .entity_id(),
        entity
    );
}
