use worth_foundational::facade::{AspectValue, CanonicalF32};
use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::{domain, foundation::WorthQueryEntityIdentity, runtime};
use worth_ui::facade::app::WorthUi;
use worth_ui::facade::declaration::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership, MeasurementConstraint, MeasurementValue, MosaicChildRule,
    MosaicClippingPosture, MosaicFocusScopeKind, MosaicHitTestPosture, MosaicMeasurementAuthority,
    MosaicOverflowBehavior, MosaicParentGrowthBehavior, MosaicRegionKindDescriptor,
    MosaicRegionKindId, MosaicRegionPersistence, MosaicRegionRole, MosaicResizePermission,
    MosaicScrollOwnership, MosaicSizingBehavior, MosaicSizingContractDescriptor,
    MosaicSizingContractId, MosaicSizingKind, MosaicSizingPersistence, MosaicViewportConstraint,
    NamedMeasurementDefinition, NamedMeasurementToken, SurfacePlacementClass,
};
use worth_ui::facade::source::{
    WorthUiFilesystemSourceProvider, WorthUiWatchedCandidateSubmission,
};
use worth_ui_query_binding::WorthUiInstalledLiveQueryView;
use worth_ui_query_binding::{
    worth_ui_domain_package, worth_ui_native_aspect_contracts, WorthUiInstalledQueryView,
    WorthUiInstalledSnapshotQueryView,
};
use worth_ui_query_binding::{
    WorthUiAdmittedQueryBindingReference, WorthUiQueryBindingPlan, WorthUiQueryViewShape,
};
use worth_ui_test_support::WorthUiApplicationBuilderCertificationExt;

type BoundBuilder = worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder;
type UnboundBuilder = worth_ui::facade::app::WorthUiApplicationBuilder<
    worth_ui::facade::app::UiChangeProfileInstalled,
    worth_ui::facade::app::UiIntentWiringSatisfied,
>;

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;

pub(crate) const ACTIVE_COMPONENT: &str = "workspace.component.query_lifecycle_active";
pub(crate) const NEXT_COMPONENT: &str = "workspace.component.query_lifecycle_next";
pub(crate) const FIRST_VIEW: &str = "inspector.first";
pub(crate) const SECOND_VIEW: &str = "inspector.second";
pub(super) const SNAPSHOT_VIEW: &str = "inspector.snapshot";
const REGION: &str = "workspace.region.query_lifecycle";
const SIZING: &str = "workspace.sizing.query_lifecycle";

pub(crate) fn application(
    first: WorthUiInstalledLiveQueryView,
    second: WorthUiInstalledLiveQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> worth_ui::facade::app::WorthUiApp {
    let binding_reference = settled_binding_reference(first.clone().into(), workspace);
    let snapshot = builder(
        first.clone().into(),
        second.clone().into(),
        &binding_reference,
    )
    .freeze()
    .expect("capability snapshot");
    builder(first.into(), second.into(), &binding_reference)
        .with_candidate_submission(submission(
            "query-lifecycle-active",
            ACTIVE_COMPONENT,
            &[FIRST_VIEW],
            snapshot.capabilities(),
        ))
        .freeze()
        .expect("source-backed Query app")
}

pub(crate) fn snapshot_application(
    first: WorthUiInstalledSnapshotQueryView,
    second: WorthUiInstalledSnapshotQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> worth_ui::facade::app::WorthUiApp {
    let binding_reference = settled_binding_reference(first.clone().into(), workspace);
    let snapshot = builder(
        first.clone().into(),
        second.clone().into(),
        &binding_reference,
    )
    .freeze()
    .expect("snapshot capability application");
    builder(first.into(), second.into(), &binding_reference)
        .with_candidate_submission(submission(
            "query-snapshot-lifecycle-active",
            ACTIVE_COMPONENT,
            &[FIRST_VIEW],
            snapshot.capabilities(),
        ))
        .freeze()
        .expect("source-backed snapshot Query app")
}

pub(super) fn mixed_live_snapshot_application(
    first: WorthUiInstalledLiveQueryView,
    second: WorthUiInstalledLiveQueryView,
    snapshot: WorthUiInstalledSnapshotQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> worth_ui::facade::app::WorthUiApp {
    let binding_reference = settled_binding_reference(first.clone().into(), workspace);
    let capabilities = builder(
        first.clone().into(),
        second.clone().into(),
        &binding_reference,
    )
    .register_query_view(snapshot.clone())
    .expect("snapshot view registration")
    .freeze()
    .expect("mixed capability application");
    builder(first.into(), second.into(), &binding_reference)
        .register_query_view(snapshot)
        .expect("snapshot view registration")
        .with_candidate_submission(submission(
            "query-mixed-lifecycle-active",
            ACTIVE_COMPONENT,
            &[FIRST_VIEW, SNAPSHOT_VIEW],
            capabilities.capabilities(),
        ))
        .freeze()
        .expect("source-backed mixed Query application")
}

pub(crate) fn submission(
    provider_id: &str,
    component: &str,
    bindings: &[&str],
    capabilities: &worth_ui::facade::diagnostics::CapabilitySnapshot,
) -> WorthUiWatchedCandidateSubmission {
    let mut source = format!("component {component} {{ region {REGION} {{ sizing {SIZING}; }} }}");
    for binding in bindings {
        source.push_str(&format!("\nbinding {binding} {{}}"));
    }
    let workspace = FilesystemContractWorkspace::new(provider_id);
    workspace.write("app/main.wui", &source);
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("production filesystem acquisition reads real Query-bound .wui bytes");
    workspace.close();
    snapshot
        .attempt_candidate_for_certification(capabilities)
        .expect("filesystem source lowers against exact capabilities")
}

pub(super) fn installed_workspace(label: &str) -> runtime::WorthQueryWorkspace {
    installed_workspace_with_measurement_authority(label).0
}

pub(crate) fn installed_workspace_with_measurement_authority(
    label: &str,
) -> (runtime::WorthQueryWorkspace, WorthQueryEntityIdentity) {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(worth_ui_native_aspect_contracts())
        .expect("native contracts")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect");
    let mut workspace = crate::request_custody::with_bootstrap_request(|execution| {
        worth_ui_query_binding::install_worth_ui_test_operation_executors(operation_live_support(
            in_memory_test_runtime()
                .with_schema(schema)
                .domain_package(worth_ui_domain_package()),
        ))
        .workspace(execution, label)
        .expect("installed Query workspace")
    });
    let measurement = insert_measurement(&mut workspace);
    (workspace, measurement)
}

pub(crate) fn insert_measurement(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> WorthQueryEntityIdentity {
    workspace
        .with_execution_request_mut(|execution, workspace| {
            workspace
                .insert(execution, "WorthUiMeasurement", |measurement| {
                    measurement
                        .set_aspect(
                            runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                                "identity.id",
                            )
                            .expect("identity touch"),
                            runtime::WorthQueryAuthoredAspectValue::string(
                                "query-lifecycle-measurement",
                            ),
                        )
                        .set_aspect(
                            runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                                "measurement.value",
                            )
                            .expect("measurement touch"),
                            runtime::WorthQueryAuthoredAspectValue::native(AspectValue::Float32(
                                CanonicalF32::from_f32(240.0),
                            )),
                        )
                })
                .expect("real replacement-lifecycle measurement insertion")
                .deltas()[0]
                .entity_identity()
                .clone()
        })
        .expect("workspace host call opens")
}

pub(crate) fn operation_live_support(
    builder: worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    [
        domain::WorthQueryConsumerSupportDimension::Live,
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportDimension::Invalidation,
        domain::WorthQueryConsumerSupportDimension::DependencyImpact,
        domain::WorthQueryConsumerSupportDimension::CollectionDelivery,
    ]
    .into_iter()
    .fold(builder, |builder, dimension| {
        builder.consumer_support_posture(
            dimension,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
    })
}

fn builder(
    first: WorthUiInstalledQueryView,
    second: WorthUiInstalledQueryView,
    binding_reference: &WorthUiAdmittedQueryBindingReference,
) -> BoundBuilder {
    BoundBuilder::new(
        unbound_builder(first, second, binding_reference),
        worth_ui_host_headless::WorthUiHeadlessHost,
    )
}

fn unbound_builder(
    first: WorthUiInstalledQueryView,
    second: WorthUiInstalledQueryView,
    binding_reference: &WorthUiAdmittedQueryBindingReference,
) -> UnboundBuilder {
    let graph_world = worth_ui::facade::graph::UiGraphWorldProfile::settled_query_binding(
        worth_ui::facade::declaration::ViewBindingId::new(FIRST_VIEW).unwrap(),
        binding_reference,
    );
    WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(graph_world)
        .register_component(component(ACTIVE_COMPONENT))
        .register_component(component(NEXT_COMPONENT))
        .register_mosaic_region_kind(
            MosaicRegionKindDescriptor::new(
                MosaicRegionKindId::new(REGION).expect("region id"),
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
                MosaicSizingContractId::new(SIZING).expect("sizing id"),
                MosaicSizingKind::fill(),
            )
            .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
            .with_resize_permission(MosaicResizePermission::user_resizable())
            .with_persistence(MosaicSizingPersistence::restorable())
            .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
            .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
            .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
            .with_named_measurement(NamedMeasurementDefinition::new(
                NamedMeasurementToken::new("workspace.measurement.query_lifecycle")
                    .expect("measurement token"),
                MeasurementValue::logical_pixels(320),
                MeasurementConstraint::between(
                    MeasurementValue::logical_pixels(200),
                    MeasurementValue::logical_pixels(640),
                ),
            )),
        )
        .register_query_view(first)
        .expect("first Query view registration")
        .register_query_view(second)
        .expect("second Query view registration")
}

fn settled_binding_reference(
    view: WorthUiInstalledQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> WorthUiAdmittedQueryBindingReference {
    let identity = view.definition().identity().clone();
    let plan = WorthUiQueryBindingPlan::default()
        .register_view(view)
        .expect("scenario registers the real installed Query view");
    let reference = plan
        .resolve_definition(&identity, WorthUiQueryViewShape::Collection)
        .expect("scenario resolves the registered Query view");
    reference
        .enter_snapshot_attempt(workspace)
        .expect("scenario enters the exact Query world")
        .prepare_snapshot_consumer(
            crate::query_consumer_kit_workspace::interactive_borrowed_collection_requirements(),
        )
        .expect("scenario prepares the exact Query consumer")
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume()
        .unwrap()
        .settle()
        .unwrap()
        .fact()
        .binding_reference()
        .clone()
}

fn component(identity: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(identity).expect("component id"),
        ComponentPropSchema::named(format!("{identity}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}
