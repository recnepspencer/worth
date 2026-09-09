use worth_ui::facade::declaration::{
    MeasurementConstraint, MeasurementValue, MosaicChildRule, MosaicClippingPosture,
    MosaicFocusScopeKind, MosaicHitTestPosture, MosaicMeasurementAuthority, MosaicOverflowBehavior,
    MosaicParentGrowthBehavior, MosaicRegionKindDescriptor, MosaicRegionKindId,
    MosaicRegionPersistence, MosaicRegionRole, MosaicResizePermission, MosaicScrollOwnership,
    MosaicSizingBehavior, MosaicSizingContractDescriptor, MosaicSizingContractId, MosaicSizingKind,
    MosaicSizingPersistence, MosaicViewportConstraint, NamedMeasurementDefinition,
    NamedMeasurementToken, SurfacePlacementClass,
};
use worth_ui::facade::interaction::UiHostInteractionIngressOutcome;
use worth_ui::facade::observation_report::{
    UiHostObservationLoss, UiHostObservationPayload, UiHostSurfacePosition,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_contract::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UiHostScrollDeltaSource,
    UiHostScrollDeltaTargetAffinity,
};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline,
};
use worth_ui_test_support::{
    UiScrollObservationCertificationOutcome, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedFrameExecutionCertificationExt, WorthUiMountedIdentityCertificationExt,
    WorthUiMountedPublicationCertificationExt, WorthUiScrollObservationCertificationExt,
};

use crate::host_observation_fixture::{batch, report, source};
use crate::mounted_application_lifecycle::published_mounted_world::presented_epoch;

const OUTER_REGION: &str = "phase315.scroll.region.outer";
const INNER_REGION: &str = "phase315.scroll.region.inner";
const SIZING: &str = "phase315.scroll.sizing";
const LARGE_SIZING: &str = "phase315.scroll.sizing.large";

pub(super) fn with_scroll_mosaic(
    builder: worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder,
) -> worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder{
    with_scroll_mosaic_ownership(builder, MosaicScrollOwnership::region_owned())
}

pub(super) fn with_surface_scroll_mosaic(
    builder: worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder,
) -> worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder{
    with_scroll_mosaic_ownership(builder, MosaicScrollOwnership::surface_owned())
}

fn with_scroll_mosaic_ownership(
    builder: worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder,
    ownership: MosaicScrollOwnership,
) -> worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder{
    builder
        .register_mosaic_region_kind(
            MosaicRegionKindDescriptor::new(
                MosaicRegionKindId::new(OUTER_REGION).expect("valid outer scroll region id"),
                MosaicRegionRole::primary(),
            )
            .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
            .with_scroll_ownership(ownership.clone())
            .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
            .with_child_rule(MosaicChildRule::accepts_regions())
            .with_persistence(MosaicRegionPersistence::restorable())
            .with_clipping(MosaicClippingPosture::clip_to_region())
            .with_hit_test(MosaicHitTestPosture::participates()),
        )
        .register_mosaic_region_kind(
            MosaicRegionKindDescriptor::new(
                MosaicRegionKindId::new(INNER_REGION).expect("valid inner scroll region id"),
                MosaicRegionRole::primary(),
            )
            .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
            .with_scroll_ownership(ownership)
            .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
            .with_child_rule(MosaicChildRule::accepts_surfaces())
            .with_allowed_surface_class(SurfacePlacementClass::primary_region())
            .with_persistence(MosaicRegionPersistence::restorable())
            .with_clipping(MosaicClippingPosture::clip_to_region())
            .with_hit_test(MosaicHitTestPosture::participates()),
        )
        .register_mosaic_sizing_contract(
            MosaicSizingContractDescriptor::new(
                MosaicSizingContractId::new(SIZING).expect("valid scroll sizing id"),
                MosaicSizingKind::fill(),
            )
            .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
            .with_resize_permission(MosaicResizePermission::user_resizable())
            .with_persistence(MosaicSizingPersistence::restorable())
            .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
            .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
            .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
            .with_named_measurement(NamedMeasurementDefinition::new(
                NamedMeasurementToken::new("phase315.scroll.measurement")
                    .expect("valid scroll measurement token"),
                MeasurementValue::logical_pixels(320),
                MeasurementConstraint::between(
                    MeasurementValue::logical_pixels(200),
                    MeasurementValue::logical_pixels(640),
                ),
            )),
        )
        .register_mosaic_sizing_contract(
            MosaicSizingContractDescriptor::new(
                MosaicSizingContractId::new(LARGE_SIZING).expect("valid large scroll sizing id"),
                MosaicSizingKind::fill(),
            )
            .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
            .with_resize_permission(MosaicResizePermission::user_resizable())
            .with_persistence(MosaicSizingPersistence::restorable())
            .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
            .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
            .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
            .with_named_measurement(NamedMeasurementDefinition::new(
                NamedMeasurementToken::new("phase315.scroll.measurement.large")
                    .expect("valid large scroll measurement token"),
                MeasurementValue::logical_pixels(480),
                MeasurementConstraint::between(
                    MeasurementValue::logical_pixels(200),
                    MeasurementValue::logical_pixels(640),
                ),
            )),
        )
}

pub(super) fn scroll_visual_source() -> String {
    FilesystemApplicationLifecycleScenario::visual_identity_source_text().replacen(
        "component visual.identity.component.hit_only {}",
        &format!(
            "component visual.identity.component.hit_only {{ region {OUTER_REGION} {{ sizing {SIZING}; region {INNER_REGION} {{ sizing {SIZING}; }} }} }}"
        ),
        1,
    )
}

pub(super) fn sibling_scroll_visual_source() -> String {
    scroll_visual_source().replacen(
        "component visual.identity.component.paint_and_hit {}",
        &format!(
            "component visual.identity.component.paint_and_hit {{ region {OUTER_REGION} {{ sizing {SIZING}; region {INNER_REGION} {{ sizing {SIZING}; }} }} }}"
        ),
        1,
    )
}

pub(super) fn mixed_extent_sibling_scroll_visual_source() -> String {
    FilesystemApplicationLifecycleScenario::visual_identity_source_text()
        .replacen(
            "component visual.identity.component.hit_only {}",
            &format!(
                "component visual.identity.component.hit_only {{ region {OUTER_REGION} {{ sizing {LARGE_SIZING}; region {INNER_REGION} {{ sizing {LARGE_SIZING}; }} }} }}"
            ),
            1,
        )
        .replacen(
            "component visual.identity.component.paint_and_hit {}",
            &format!(
                "component visual.identity.component.paint_and_hit {{ region {OUTER_REGION} {{ sizing {SIZING}; region {INNER_REGION} {{ sizing {SIZING}; }} }} }}"
            ),
            1,
        )
}

pub(super) fn reduced_sibling_scroll_visual_source() -> String {
    scroll_visual_source().replacen(
        "component visual.identity.component.paint_and_hit {}",
        &format!(
            "component visual.identity.component.paint_and_hit {{ region {OUTER_REGION} {{ sizing {SIZING}; }} }}"
        ),
        1,
    )
}

pub(super) fn publish_predecessor(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let prepared = prepare_frame(session);
    assert!(matches!(
        session
            .present_prepared_mounted_frame(prepared, UiPresentationDeadline::at_tick(1_000), 0,),
        UiMountedFrameOutcome::Published(_)
    ));
}

pub(super) fn publish_with_hit_coordinate(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    binding: worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    scroll_target: worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
) -> (
    crate::mounted_application_lifecycle::published_mounted_world::PresentedObservationBasis,
    UiHostSurfacePosition,
) {
    let prepared = prepare_frame(session);
    let row = prepared.surfaces()[0]
        .projection()
        .hit_tests()
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == scroll_target)
        .copied()
        .expect("mosaic hit-test posture emits an exact coordinate target");
    let bounds = row.clip_bounds();
    let unit = UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32;
    let coordinate = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + 1.0) * unit).round() as i64,
        ((bounds.y() + 1.0) * unit).round() as i64,
    );
    let publication = match session.present_prepared_mounted_frame(
        prepared,
        UiPresentationDeadline::at_tick(1_000),
        0,
    ) {
        UiMountedFrameOutcome::Published(publication) => publication,
        _ => panic!("scripted mosaic frame must publish"),
    };
    let host_surface =
        session.inspect_mounted_identity().surface_bindings()[0].host_surface_identity();
    (
        crate::mounted_application_lifecycle::published_mounted_world::PresentedObservationBasis {
            host_surface,
            frame: publication.frame(),
            epoch: presented_epoch(session, publication.frame(), binding),
            instance: row.mounted_instance(),
            receipt: row.node_receipt(),
        },
        coordinate,
    )
}

fn prepare_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("current framework turn is available")
        .into_execution()
        .unwrap_or_else(|_| panic!("mosaic-authored turn produces execution"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
        .expect("mosaic-authored frame prepares")
}

pub(super) fn admit_scroll(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    binding: worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    basis: &crate::mounted_application_lifecycle::published_mounted_world::PresentedObservationBasis,
    sequence: u64,
    phase: UiHostScrollDeltaPhase,
    target: UiHostScrollDeltaTargetAffinity,
    x_subpixels: i64,
    y_subpixels: i64,
) -> UiScrollObservationCertificationOutcome {
    let payload = UiHostObservationPayload::ScrollDelta {
        source: UiHostScrollDeltaSource::PointerWheel,
        phase,
        precision: UiHostScrollDeltaPrecision::Pixel,
        target,
        x_subpixels,
        y_subpixels,
    };
    let batch = batch(
        source(session, binding, basis),
        (sequence, sequence),
        UiHostObservationLoss::Complete,
        vec![report(sequence, payload, basis)],
    );
    let UiHostInteractionIngressOutcome::Applied(receipt) =
        session.admit_host_interaction_batch(batch)
    else {
        panic!("current scroll observation must pass host ingress")
    };
    let outcomes = receipt.scroll_observations_for_certification();
    assert_eq!(outcomes.len(), 1);
    outcomes[0]
}
