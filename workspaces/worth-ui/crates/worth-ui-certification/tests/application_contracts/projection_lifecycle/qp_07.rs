use worth_ui_query_binding::{
    UiPresentProjection, UiProjectionAvailability, UiProjectionObservation,
    UiProjectionUnavailableKind, WorthUiScalarProjectionHostPlan,
    WorthUiScalarProjectionSourceRecord,
};

#[test]
fn every_rebind_safe_point_disposes_or_returns_its_exact_owner() {
    crate::observation_rebind::lifecycle_cleanup::prove_all_projection_safe_point_cleanup();
}

#[test]
fn scalar_product_owner_closes_pending_revalidation_and_receipt_residue() {
    let plan = WorthUiScalarProjectionHostPlan::prepare().expect("product plan prepares");
    let (request, completion) = plan.into_parts();
    let installation =
        worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(request.generation(), request.into_packages())
            .expect("host installs the exact admitted packages");
    let installed = completion
        .complete(installation)
        .expect("binding completion opens the production Query owner");
    let initial = installed.into_initial_advance();
    assert_unavailable(initial.observation(), UiProjectionUnavailableKind::Pending);
    let owner = publish(initial);

    let current = owner
        .advance(
            WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("first native source record"),
        )
        .expect("pending owner reaches current");
    assert_current(current.observation(), "ONLINE");
    let owner = publish(current);

    let updated = owner
        .advance(
            WorthUiScalarProjectionSourceRecord::new("UPDATED", 2)
                .expect("replacement native source record"),
        )
        .expect("current owner revalidates and republishes");
    assert_current(updated.observation(), "UPDATED");
    let closed = publish(updated)
        .close()
        .expect("the exact product owner closes");

    assert!(closed.owner_terminal());
    assert_eq!(closed.live_source_count(), 0);
    assert_eq!(closed.live_attempt_count(), 0);
    assert_eq!(closed.live_resource_count(), 0);
    assert_eq!(closed.live_consumer_lease_count(), 0);
    assert_eq!(closed.retained_projection_count(), 0);
    assert_eq!(closed.projection_receipt_count(), 0);
}

#[test]
fn collection_continuation_reset_and_hostile_denials_close_every_owner() {
    crate::collection_projection::qp_04::mutations::
        complete_partial_and_continuation_postures_come_from_real_query_results();
    crate::collection_projection::qp_04::mutations::
        continuation_completion_and_explicit_reset_are_preserved();
    crate::collection_projection::qp_04::hostile::
        hostile_query_patches_return_exact_denials_and_mint_no_ui_effect();
}

fn publish(
    advance: worth_ui_query_binding::WorthUiScalarProjectionAdvance,
) -> worth_ui_query_binding::WorthUiScalarProjectionLiveOwner {
    let (observation, completion) = advance.into_parts();
    let observation = match observation {
        UiProjectionObservation::Scalar(observation) => observation,
        UiProjectionObservation::Collection(_) => {
            panic!("the scalar product owner cannot issue collection evidence")
        }
        UiProjectionObservation::ApplicationScalar(_) => {
            panic!("the certification scalar owner cannot issue authored application evidence")
        }
    };
    completion
        .admit_publication(observation)
        .expect("the exact fact readmits its move-only owner")
}

fn assert_unavailable(
    observation: &UiProjectionObservation,
    expected: UiProjectionUnavailableKind,
) {
    let UiProjectionObservation::Scalar(observation) = observation else {
        panic!("the scalar product owner cannot issue collection evidence")
    };
    assert!(matches!(
        observation.fact().availability(),
        UiProjectionAvailability::Unavailable(unavailable) if unavailable.kind() == expected
    ));
}

fn assert_current(observation: &UiProjectionObservation, expected: &str) {
    let UiProjectionObservation::Scalar(observation) = observation else {
        panic!("the scalar product owner cannot issue collection evidence")
    };
    assert!(matches!(
        observation.fact().availability(),
        UiProjectionAvailability::Present(UiPresentProjection::Current(value))
            if value.as_str() == expected
    ));
}
