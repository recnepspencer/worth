use crate::capability::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership, SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass, UiIntent, UiIntentAcceptedInteractions, UiIntentId, UiIntentPayload,
    UiIntentPayloadFieldSet, UiIntentPayloadProjection, UiIntentPayloadProjectionViolation,
    UiIntentProductConsequenceFamilies, UiIntentProductOutcome, UiIntentRuntimeServiceDestination,
    UiIntentSchema, UiSemanticInteractionFamily,
};
use crate::facade::WorthUi;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};

pub(super) const AUTHORED_PORTAL_SOURCE: &str = r#"
surface workspace.surface.overlay {}
intent workspace.intent.open {
    definition workspace.intent.open
    interaction activate
    operability workspace.intent.open.operability
        mutability-application-boolean workspace.intent.open.mutable
        readiness-application-boolean workspace.intent.open.ready
        policy-application-boolean workspace.intent.open.policy
    confirmation workspace.intent.open.confirmation not-required
    concurrency target-route-single-flight
    consequences mounted-posture
}
portal overlay.menu {
    surface workspace.surface.overlay
    anchor workspace.anchor
    layer transient
    dismiss escape
    focus first_enabled
    motion system_popover
}
component workspace.component.overlay {
    region workspace.region.primary {
        sizing workspace.sizing.mosaic_support;
    }
    interaction activate routes workspace.intent.open opens portal overlay.menu;
}
"#;

struct AuthoredPortalPayload;
struct AuthoredPortalOutcome;
struct AuthoredPortalIntent;

impl UiIntentPayload for AuthoredPortalPayload {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.authored_portal.payload", 1);
    const FIELDS: UiIntentPayloadFieldSet = UiIntentPayloadFieldSet::EMPTY;

    fn project(
        _fields: &mut UiIntentPayloadProjection<Self>,
    ) -> Result<Self, UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

impl UiIntentProductOutcome for AuthoredPortalOutcome {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.authored_portal.outcome", 1);
    const CONSEQUENCE_FAMILIES: UiIntentProductConsequenceFamilies =
        UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> crate::capability::UiIntentProductConsequences {
        crate::capability::UiIntentProductConsequences::none()
    }
}

impl UiIntent for AuthoredPortalIntent {
    type Payload = AuthoredPortalPayload;
    type ProductOutcome = AuthoredPortalOutcome;

    const ID: UiIntentId = UiIntentId::stable("workspace.intent.open");
    const ACCEPTED_INTERACTIONS: UiIntentAcceptedInteractions =
        UiIntentAcceptedInteractions::new(&[UiSemanticInteractionFamily::Activate]);
}

pub(super) fn authored_overlay_builder(
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    authored_overlay_builder_with_appearance_contract(
        worth_ui_dsl::UiAppearanceAspectContract::component(
            [worth_ui_dsl::UiAppearanceAspect::Background],
            [],
        )
        .expect("overlay component appearance contract"),
    )
}

pub(super) fn authored_overlay_builder_with_appearance_contract(
    contract: worth_ui_dsl::UiAppearanceAspectContract,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    authored_overlay_builder_with_appearance_contract_and_region(
        contract,
        crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region(),
    )
}

pub(super) fn authored_overlay_builder_with_appearance_contract_and_region(
    contract: worth_ui_dsl::UiAppearanceAspectContract,
    region: crate::capability::MosaicRegionKindDescriptor,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    authored_overlay_builder_with_component_and_region(
        ComponentDescriptor::new(
            ComponentId::new("workspace.component.overlay").expect("valid component identity"),
            ComponentPropSchema::named("workspace.component.overlay.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        )
        .with_hit_test(
            crate::capability::ComponentHitTestContract::allocation_bounds(
                crate::capability::ComponentHitTestOrder::front_to_back(0),
                crate::capability::ComponentAllocationMeasurementContract::viewport_inset(
                    crate::capability::ComponentViewportInset::symmetric(0, 0),
                ),
            ),
        )
        .with_surface_paint_order(0)
        .with_appearance_aspect_contract(contract)
        .expect("component appearance contract should admit"),
        region,
    )
}

pub(super) fn authored_overlay_builder_with_component(
    component: ComponentDescriptor,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    authored_overlay_builder_with_component_and_region(
        component,
        crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region(),
    )
}

fn authored_overlay_builder_with_component_and_region(
    component: ComponentDescriptor,
    region: crate::capability::MosaicRegionKindDescriptor,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(
                "workspace.intent.open.mutable",
            )
            .expect("authored Portal mutability fact identity"),
            true,
        )
        .expect("authored Portal mutability fact should register")
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(
                "workspace.intent.open.ready",
            )
            .expect("authored Portal readiness fact identity"),
            true,
        )
        .expect("authored Portal readiness fact should register")
        .register_intent_boolean_fact(
            crate::declaration::UiIntentApplicationFact::boolean(
                "workspace.intent.open.policy",
            )
            .expect("authored Portal policy fact identity"),
            true,
        )
        .expect("authored Portal policy fact should register")
        .register_runtime_service_intent_definition(
            crate::capability::UiIntentDefinition::<AuthoredPortalIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::OpenPortal,
            ),
        )
        .expect("authored Portal runtime-service definition should register")
        .register_component(component)
        .register_surface(SurfaceDescriptor::new(
            SurfaceId::new("workspace.surface.overlay").expect("valid surface identity"),
            SurfaceKind::overlay_content(),
            ComponentId::new("workspace.component.overlay").expect("valid component identity"),
            SurfacePlacementClass::overlay_layer(),
            SurfaceStateClass::restorable(),
        ))
        .register_mosaic_region_kind(region)
        .register_mosaic_sizing_contract(
            crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_sizing(),
        )
}

pub(super) fn authored_overlay_session() -> crate::facade::WorthUiActiveApplicationSession {
    let capability_app = authored_overlay_builder()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("authored Portal capabilities should prepare");
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("authored-portal-runtime")
            .with_file("app/main.wui", AUTHORED_PORTAL_SOURCE),
        [WorthUiWatcherEvent::provider_revision(
            "authored-portal-runtime",
        )],
        capability_app.capabilities(),
    );
    authored_overlay_builder()
        .with_candidate_submission(submission)
        .freeze()
        .map(|app| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_recorder(
                app,
                worth_ui_host_headless::WorthUiHeadlessRecorder::with_viewport_extent(
                    worth_ui_host_headless::UiHeadlessRecorderCapacity::production_default(),
                    worth_ui_host_contract::UiViewportExtentObservation {
                        width: 640.0,
                        height: 480.0,
                    },
                ),
            )
        })
        .expect("authored Portal source should prepare")
        .launch()
        .expect("authored Portal source should launch")
}

pub(super) fn portal_target(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: crate::facade::mounted::UiSemanticSurfaceIdentity,
) -> (
    crate::graph::UiGraphNodeIdentity,
    crate::facade::mounted::UiMountedInstanceIdentity,
) {
    let (graph_node, authored_identity) = {
        let graph = session.graph();
        graph
            .node_identities()
            .filter_map(|identity| {
                let lookup = graph.lookup().graph_node(identity)?;
                let authored = lookup
                    .value()
                    .declaration_identity()
                    .authored_semantic_name()
                    .to_owned();
                authored
                    .starts_with("component:")
                    .then_some((identity, Box::<str>::from(authored)))
            })
            .next()
            .expect("authored source should admit one component graph node")
    };
    session
        .register_application_semantic_text(authored_identity, graph_node)
        .expect("Portal target semantic text should be admitted");
    let mounted_node = session
        .mounted_graph_node(graph_node)
        .expect("authored component should have a mounted graph handle");
    let mounted = session
        .mount_instance(mounted_node, surface)
        .expect("authored Portal target should mount on its declared surface");
    (graph_node, mounted)
}

pub(super) fn presentation() -> worth_ui_host_contract::UiHostObservationPresentationBasis {
    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound()
            .expect("Portal presentation host identity capacity"),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound()
            .expect("Portal presentation frame identity capacity"),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
            .expect("Portal presentation binding identity capacity"),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    )
}
