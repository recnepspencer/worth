use super::*;
use crate::filesystem_mounted_world::establish_allocation;
use worth_ui::facade::declaration::{
    MeasurementConstraint, MeasurementValue, MosaicChildRule, MosaicClippingPosture,
    MosaicFocusScopeKind, MosaicHitTestPosture, MosaicMeasurementAuthority, MosaicOverflowBehavior,
    MosaicParentGrowthBehavior, MosaicRegionKindDescriptor, MosaicRegionKindId,
    MosaicRegionPersistence, MosaicRegionRole, MosaicResizePermission, MosaicScrollOwnership,
    MosaicSizingBehavior, MosaicSizingContractDescriptor, MosaicSizingContractId, MosaicSizingKind,
    MosaicSizingPersistence, MosaicViewportConstraint, NamedMeasurementDefinition,
    NamedMeasurementToken, SurfacePlacementClass,
};
use worth_ui::facade::service::UiSelectionPolicy;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedIdentityCertificationExt,
};

#[path = "scroll_selection_input.rs"]
mod scroll_selection_input;
pub(in crate::intent) use scroll_selection_input::routed_scroll_selection_input;
#[path = "scroll_portal_geometry.rs"]
mod geometry;

const SCROLL_REGION: &str = "phase315.selection.scroll_region";
const SCROLL_SIZING: &str = "phase315.selection.scroll_sizing";
const SCROLL_MEASUREMENT: &str = "phase315.selection.scroll_measurement";

pub(in crate::intent) fn launch_scroll_portal<I: UiIntent>(
    input: WorthUiRustAuthoredArtifactInput,
    projection: PayloadProjectionRegistration,
    facts: PayloadApplicationFacts,
    host: WorthUiHeadlessRecorder,
) -> PayloadWorld {
    let projection_slot = projection_slot(&projection);
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-315-selection-portal-world");
    let builder = scenario
        .portal_semantic_text_action_application_builder(host)
        .register_mosaic_region_kind(
            MosaicRegionKindDescriptor::new(
                MosaicRegionKindId::new(SCROLL_REGION).expect("valid selection scroll region"),
                MosaicRegionRole::primary(),
            )
            .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
            .with_scroll_ownership(MosaicScrollOwnership::region_owned())
            .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
            .with_child_rule(MosaicChildRule::accepts_surfaces())
            .with_allowed_surface_class(SurfacePlacementClass::primary_region())
            .with_persistence(MosaicRegionPersistence::restorable())
            .with_clipping(MosaicClippingPosture::clip_to_region())
            .with_hit_test(MosaicHitTestPosture::participates()),
        )
        .register_mosaic_sizing_contract(
            MosaicSizingContractDescriptor::new(
                MosaicSizingContractId::new(SCROLL_SIZING).expect("valid selection scroll sizing"),
                MosaicSizingKind::fill(),
            )
            .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
            .with_resize_permission(MosaicResizePermission::user_resizable())
            .with_persistence(MosaicSizingPersistence::restorable())
            .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
            .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
            .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
            .with_named_measurement(NamedMeasurementDefinition::new(
                NamedMeasurementToken::new(SCROLL_MEASUREMENT)
                    .expect("valid selection scroll measurement"),
                MeasurementValue::logical_pixels(320),
                MeasurementConstraint::between(
                    MeasurementValue::logical_pixels(200),
                    MeasurementValue::logical_pixels(640),
                ),
            )),
        )
        .register_runtime_service_intent_definition(UiIntentDefinition::<I>::runtime_service(
            UiIntentRuntimeServiceDestination::OpenPortal,
        ))
        .expect("typed selection portal definition registers")
        .with_selection_policy_defaults(UiSelectionPolicy::multiple());
    let builder = register_projection(builder, projection);
    let builder = register_facts(builder, facts);
    let application = builder
        .with_rust_authored_input(input)
        .freeze()
        .expect("selection portal world compiles through production preparation");
    assert_eq!(
        application.service_policy_plan().selection(),
        Some(UiSelectionPolicy::multiple()),
        "SelectionCommit declarations demand the owner and preserve public policy defaults"
    );
    let nodes = component_graph_nodes(&application);
    let mut session = application
        .launch()
        .expect("Selection Portal application launches");
    let surface = session.create_declared_semantic_surface(SURFACE).unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            crate::mounted_application_lifecycle::known_empty_surface_world::profile(1),
        )
        .unwrap();
    for node in nodes {
        let handle = session.mounted_graph_node(node).unwrap();
        session.mount_instance(handle, surface).unwrap();
    }
    establish_allocation(&mut session, 3);
    geometry::complete(&mut session, surface);
    let prepared = session
        .prepare_application_presentation_frame(
            worth_ui_runtime::facade::mounted::UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("Selection Portal prepares from its completed layout");
    PayloadWorld {
        interaction: InteractionWorld::from_prepared_frame(session, prepared),
        projection_slot,
    }
}

fn bind_portal_operability<I: UiIntent>(
    declaration: UiIntentDeclaration<I>,
) -> worth_ui_dsl::WorthUiIntentDeclarationSpec {
    let fact = operability_fact();
    declaration
        .operability_from(
            UiIntentOperabilityContract::new(
                OPERABILITY_CONTRACT,
                UiIntentMutabilitySource::application_fact(&fact),
                UiIntentReadinessSource::application_fact(&fact),
                UiIntentPolicySource::application_fact(&fact),
            )
            .expect("payload operability contract identity is valid"),
        )
        .confirmation(
            UiIntentConfirmationContract::not_required(CONFIRMATION_POLICY)
                .expect("payload confirmation policy identity is valid"),
        )
        .concurrency(UiIntentConcurrencyScope::TargetRouteSingleFlight)
        .consequences(UiIntentConsequenceContract::mounted_posture())
        .into_dsl_spec()
}
