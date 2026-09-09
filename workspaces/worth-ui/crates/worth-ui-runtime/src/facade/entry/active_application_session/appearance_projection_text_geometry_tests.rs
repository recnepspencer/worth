use super::{
    assert_text_damage_transition, fixture, prepare, set_text, text_candidate, text_contract,
};
use worth_ui_host_contract::*;

#[test]
fn allocation_resize_routes_foreground_without_semantic_invalidation_and_survives_retry() {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let _ = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "geometry-routing-initial");
    let initial = prepare(&mut session);
    let old = text_candidate(&initial, adopted, 10);
    assert_eq!(old.bounds().width(), 800.0);
    fixture::publish(&mut session, &host, initial, 1);

    host.set_viewport_extent([400.0, 600.0]);
    host.advance_viewport_environment();
    remeasure_viewport(&mut session);
    // No owner observation, source close, text edit, or theme change accompanies
    // the admitted allocation change. Dropping preparation must not consume it.
    let mut abandoned = prepare(&mut session);
    assert!(abandoned
        .appearance_invalidation_batch()
        .unwrap()
        .is_physical_input_only());
    let resized = text_candidate(&abandoned, adopted, 10);
    assert_eq!(resized.bounds().width(), 400.0);
    assert_eq!(resized.mounted_instance(), old.mounted_instance());
    assert_eq!(resized.foregrounds(), old.foregrounds());
    assert_text_damage_transition(&abandoned, UiAppearanceTextDamageTransition::Replace);
    let (_, records) = abandoned
        .lower_appearance(
            UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            None,
        )
        .into_parts();
    assert!(!records.is_empty());
    for record in records {
        let crate::runtime::appearance::UiAppearanceInspectionRecord::Projection {
            consumers_selected,
            ..
        } = record
        else {
            panic!("geometry refresh must not deny");
        };
        assert_eq!(
            consumers_selected, 0,
            "retained physical refresh does not select semantic consumers"
        );
    }
    drop(abandoned);

    let retry = prepare(&mut session);
    let retried = text_candidate(&retry, adopted, 10);
    assert_eq!(retried.bounds(), resized.bounds());
    assert_eq!(retried.clip_bounds(), resized.clip_bounds());
    assert_text_damage_transition(&retry, UiAppearanceTextDamageTransition::Replace);
    fixture::publish(&mut session, &host, retry, 2);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
}

fn remeasure_viewport(session: &mut crate::facade::WorthUiActiveApplicationSession) {
    let witnesses = session.viewport_measurement_witnesses();
    assert!(!witnesses.is_empty());
    let capability = session.host_measurement_capability();
    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let inputs = witnesses
        .iter()
        .map(|authority| {
            crate::facade::WorthUiHostMeasurementSessionInput::new(
                authority.request_identity(),
                UiMeasurementEvidenceFamily::ViewportExtent,
                crate::host::UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
                authority.evidence_generation(),
                crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(
                    assumptions,
                ),
            )
        })
        .collect();
    session
        .settle_mounted_host_measurements(Some((capability, inputs)))
        .unwrap_or_else(|_| {
            panic!("current viewport witnesses must settle through measurement admission")
        });
}
