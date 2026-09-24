use worth_signal::facade::AspectMask;

use super::super::declarations::{UiNativePhysicalSignalAspect, UiNativePhysicalSignalOperation};
use super::super::{
    UiNativePhysicalPresentationBasis, UiNativePhysicalSignalOwner, UiNativePhysicalSignalWork,
};

#[test]
fn repeated_same_basis_requests_have_distinct_graph_carried_sequence_scopes() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let pins =
        worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
    let basis = UiNativePhysicalPresentationBasis::test_with_host_session(7);
    let first = owner.admit_atlas_planning(basis, &[], pins).unwrap();
    let first_performed = owner.runtime.worker().unwrap().last_performed().unwrap();
    let second = owner.admit_atlas_planning(basis, &[], pins).unwrap();
    let second_performed = owner.runtime.worker().unwrap().last_performed().unwrap();

    assert_eq!(first.basis_digest(), second.basis_digest());
    assert_ne!(first.sequence(), second.sequence());
    assert_ne!(
        first_performed
            .locality()
            .subscription(UiNativePhysicalSignalAspect::WorkIdentity),
        second_performed
            .locality()
            .subscription(UiNativePhysicalSignalAspect::WorkIdentity)
    );
    assert!(second_performed.fact_revision() > first_performed.fact_revision());
    let observation = owner.observation();
    assert_eq!(
        observation.performed_request_sequence,
        Some(second.sequence())
    );
    assert_eq!(
        observation.performed_fact_revision,
        Some(second_performed.fact_revision())
    );
    assert_eq!(observation.performed_read_scopes, 5);
    assert_eq!(observation.active_requests, 2);
    assert_eq!(observation.pending_wakes, 2);
    assert_eq!(
        owner.take_ready_atlas_planning(first).unwrap().work(),
        UiNativePhysicalSignalWork::AtlasPlanning(first)
    );
    assert_eq!(
        owner.take_ready_atlas_planning(second).unwrap().work(),
        UiNativePhysicalSignalWork::AtlasPlanning(second)
    );
}

#[test]
fn performed_scope_observation_convicts_a_deleted_demand_axis() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let pins =
        worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
    owner
        .admit_atlas_planning(UiNativePhysicalPresentationBasis::test(), &[], pins)
        .unwrap();
    let performed = owner.runtime.worker().unwrap().last_performed().unwrap();
    let reads = owner.declarations().resources
        [UiNativePhysicalSignalOperation::AtlasUpload.index()]
    .reads();
    let demand = UiNativePhysicalSignalAspect::Demand;
    let mutated = AspectMask::from_bits(reads.bits() & !(1 << demand.index()));
    let admitted_scopes = performed.locality().scopes_for(reads);
    let mutated_scopes = performed.locality().scopes_for(mutated);

    assert!(admitted_scopes[demand.index()].is_some());
    assert!(mutated_scopes[demand.index()].is_none());
    assert_eq!(performed.read_scopes(), 5);
    assert_eq!(mutated_scopes.iter().flatten().count(), 4);
    assert_eq!(owner.observation().performed_read_scopes, 5);
}

#[test]
fn a_route_entry_without_graph_carried_currentness_cannot_begin_work() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let pins =
        worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
    let identity = owner
        .admit_atlas_planning(UiNativePhysicalPresentationBasis::test(), &[], pins)
        .unwrap();
    let work = UiNativePhysicalSignalWork::AtlasPlanning(identity);
    let token = owner
        .route
        .token_for(owner.runtime_identity, work)
        .expect("the routing mirror still contains the request");
    assert!(owner
        .runtime
        .worker_mut()
        .unwrap()
        .graph
        .remove_current(work, token.handle()));
    assert!(owner.route.token_for(owner.runtime_identity, work).is_ok());
    assert!(owner.begin_work(work).is_err());
}
