use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentSemanticTextContract, ComponentStateOwnership,
    ComponentStaticPaintContract, ComponentStaticPaintOrder, MosaicChildRule,
    MosaicClippingPosture, MosaicFocusScopeKind, MosaicHitTestPosture, MosaicRegionKindDescriptor,
    MosaicRegionKindId, MosaicRegionPersistence, MosaicRegionRole, MosaicScrollOwnership,
    MosaicSizingBehavior, ThemeColorValue, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId,
    ThemeTokenSource, ThemeTokenValue,
};
use worth_ui::facade::measurement_exchange::{
    UiMeasurementEvidenceFamily, UiViewportExtentObservation, UiViewportExtentRequest,
};
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiChangeProfile, UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui_dsl::{
    WorthUiArtifactInputBodyAtom, WorthUiProjectionLifecycle, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_host_contract::UiSemanticTextSlot;
use worth_ui_host_headless::{
    UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect, WorthUiHeadlessRecorder,
};
use worth_ui_query_binding::{
    UiProjectionFieldRequirement, UiProjectionObservation, UiScalarProjectionRegistration,
    WorthUiQueryWorkspaceExt,
};
use worth_ui_runtime::facade::entry::UiMountedAllocationMeasurementRequest;
use worth_ui_runtime::facade::host::{
    UiHostMeasurementAssumptionProfile, UiHostMeasurementNeed,
    UiHostMeasurementNormalizationContext,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedRgba8, UiSurfaceBindingCoordinatePosture,
    UiSurfaceBindingProfile,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedAllocationCertificationExt,
    WorthUiMountedIdentityCertificationExt,
};

use crate::projection_lifecycle::support::ScalarLifecycleWorld;

pub(crate) const ACTIVE_COMPONENT: &str = "platform.pulse.projected_status";
pub(super) const CANDIDATE_COMPONENT: &str = "platform.pulse.projected_status_candidate";
pub(super) const PROJECTION: &str = "platform.pulse.status";
pub(crate) const TEXT_COLOR: &str = "theme.platform_pulse.projected_status.text";
pub(crate) const STATUS_REGION: &str = "platform.pulse.region.status_shell";

#[test]
fn real_query_scalar_publishes_same_generation_semantic_text_to_headless_host() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 320.0,
            height: 96.0,
        },
    );
    let (mut query, completion) = ScalarLifecycleWorld::standard(NodeId::new(31360, 0), "Ready");
    let registration = scalar_registration(&query);
    let mut session = projection_app(registration, recorder.clone())
        .launch()
        .expect("projection application launches");
    let mounted_instances = mount_and_allocate(&mut session);
    let active_generation = session.generation_identity().clone();

    let pending = query.initial().into_fact_and_predecessor().0;
    let current = query.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    let current_fact = current.into_fact_and_predecessor().0;
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(UiProjectionObservation::Scalar(
        current_fact.into_observation(),
    ))
    .unwrap();
    let admitted = turn.seal().unwrap();
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("the real Query value changes mounted presentation"),
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap();
    assert!(
        plan.basis().candidate_generation() == &active_generation,
        "Query-only planning must retain the post-allocation application generation"
    );
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(313))
        .expect("same-generation content rebind prepares");
    assert!(
        prepared.candidate_generation() == &active_generation,
        "prepared Query content must not mint an application successor"
    );
    let receipt = match prepared.execute(1) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("same-generation Query content must publish atomically"),
    };

    assert!(receipt.prior_generation() == &active_generation);
    assert!(receipt.active_generation() == &active_generation);
    assert!(receipt.application_publication().is_none());
    let mounted = receipt
        .mounted_publication()
        .expect("content rebind publishes one mounted frame");
    assert!(mounted.generation() == &active_generation);
    assert!(session
        .current_mounted_publication()
        .is_some_and(|current| current.frame() == mounted.frame()));

    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    let transcript = &transcripts[0];
    assert_eq!(transcript.frame(), mounted.frame());
    assert_eq!(transcript.semantic_text().len(), 2);
    let value = transcript
        .semantic_text()
        .iter()
        .find(|row| row.slot() == UiSemanticTextSlot::Value)
        .expect("the current scalar value has a semantic row");
    let posture = transcript
        .semantic_text()
        .iter()
        .find(|row| row.slot() == UiSemanticTextSlot::Posture)
        .expect("the Query posture has a semantic row");
    assert_eq!(value.text(), "Ready");
    assert_eq!(posture.text(), "CURRENT");
    assert_eq!(value.foregrounds().len(), 1);
    assert_eq!(posture.foregrounds().len(), 1);
    assert_eq!(
        value.foregrounds()[0].color(),
        UiMountedRgba8::new(255, 255, 255, 255)
    );
    assert_eq!(
        posture.foregrounds()[0].color(),
        value.foregrounds()[0].color()
    );
    assert_eq!(value.mounted_instance(), posture.mounted_instance());
    assert!(mounted_instances.contains(&value.mounted_instance()));
    assert_eq!(value.content_generation(), posture.content_generation());
    assert!(value.collection_row().is_none());
    assert!(posture.collection_row().is_none());
    assert_eq!(
        transcript.unperformed_effects(),
        &[UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: 1,
            portal_overlay_count: 0,
            semantic_text_count: 2,
            preview_node_count: 0,
        }]
    );

    drop(receipt);
    let shutdown = session.shutdown();
    assert!(shutdown.rebind().is_empty());
    assert!(shutdown.mounted_presentation().is_empty());
}

pub(crate) fn scalar_registration(world: &ScalarLifecycleWorld) -> UiScalarProjectionRegistration {
    let domain = world
        .workspace
        .worth_ui()
        .expect("Worth UI Query domain is installed");
    UiScalarProjectionRegistration::text(
        domain
            .projection_view(PROJECTION)
            .expect("Platform Pulse projection view is installed"),
        UiProjectionFieldRequirement::query_text_status(),
    )
}

pub(super) fn projection_app(
    registration: UiScalarProjectionRegistration,
    recorder: WorthUiHeadlessRecorder,
) -> worth_ui::facade::app::WorthUiApp {
    let module = projection_module(ACTIVE_COMPONENT);
    worth_ui::facade::app::WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .register_component(component_descriptor(ACTIVE_COMPONENT))
        .register_component(component_descriptor(CANDIDATE_COMPONENT))
        .register_mosaic_region_kind(status_region_descriptor())
        .register_theme_token(text_token_descriptor())
        .register_scalar_projection(registration)
        .expect("product scalar projection registers")
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .map(|application| {
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_recorder(
                application,
                recorder,
            )
        })
        .expect("projection application freezes")
}

pub(super) fn projection_module(component: &str) -> WorthUiRustAuthoredArtifactInputModule {
    projection_module_with_body(
        component,
        [
            WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier(PROJECTION.to_owned()),
        ],
    )
}

pub(super) fn projection_module_with_additional_token(
    component: &str,
    token: &str,
    value: &str,
) -> WorthUiRustAuthoredArtifactInputModule {
    projection_module(component).with_token(token, value)
}

pub(super) fn projection_module_with_region(
    component: &str,
    region: &str,
) -> WorthUiRustAuthoredArtifactInputModule {
    projection_module_with_body(
        component,
        [
            WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier(PROJECTION.to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier("region".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier(region.to_owned()),
            WorthUiArtifactInputBodyAtom::LeftBrace,
            WorthUiArtifactInputBodyAtom::RightBrace,
        ],
    )
}

fn projection_module_with_body<const N: usize>(
    component: &str,
    body: [WorthUiArtifactInputBodyAtom; N],
) -> WorthUiRustAuthoredArtifactInputModule {
    WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_component_body_atoms_and_authored_identity(
            component,
            "platform-pulse-projected-status-component",
            body,
        )
        .with_token(TEXT_COLOR, "#ffffff")
        .try_with_query_scalar_text(
            PROJECTION,
            PROJECTION,
            "status",
            WorthUiProjectionLifecycle::Live,
        )
        .unwrap()
}

pub(crate) fn component_descriptor(identity: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(identity).unwrap(),
        ComponentPropSchema::named(format!("{identity}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_static_paint(
        ComponentStaticPaintContract::opaque_fill(
            ThemeTokenId::new(TEXT_COLOR).unwrap(),
            ComponentStaticPaintOrder::back_to_front(0),
        ),
        ComponentAllocationMeasurementContract::fill_viewport(),
    )
    .with_semantic_text(ComponentSemanticTextContract::body_default(
        ThemeTokenId::new(TEXT_COLOR).unwrap(),
        1,
    ))
}

pub(crate) fn status_region_descriptor() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new(STATUS_REGION).unwrap(),
        MosaicRegionRole::status(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
    .with_scroll_ownership(MosaicScrollOwnership::region_owned())
    .with_focus_scope(MosaicFocusScopeKind::status_scope())
    .with_child_rule(MosaicChildRule::leaf_only())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(MosaicClippingPosture::clip_to_region())
    .with_hit_test(MosaicHitTestPosture::participates())
}

pub(crate) fn text_token_descriptor() -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(TEXT_COLOR).unwrap(),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(ThemeColorValue::hex("#ffffff").unwrap()),
    )
}

pub(crate) fn mount_and_allocate(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> Vec<worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity> {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            UiSurfaceBindingProfile::new(
                1_000,
                UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let nodes = session.graph().node_identities().collect::<Vec<_>>();
    let mounted_instances = nodes
        .into_iter()
        .map(|node| {
            let handle = session.mounted_graph_node(node).unwrap();
            session.mount_instance(handle, surface).unwrap()
        })
        .collect::<Vec<_>>();
    let capability = session.host_measurement_capability();
    let assumptions = UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    session
        .establish_mounted_allocation_catalog(
            1,
            [UiMountedAllocationMeasurementRequest::new(
                UiMeasurementEvidenceFamily::ViewportExtent,
                UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
                UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
            )],
        )
        .expect("real host viewport establishes mounted allocation");
    mounted_instances
}
