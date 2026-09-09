use worth_ui::facade::declaration::UiDeclarationStructuralRole;
use worth_ui::facade::graph::{UiGraphLookupCostClass, UiGraphNodeIdentity};
use worth_ui::facade::measurement_exchange::{
    UiMeasurementEvidenceFamily, UiViewportExtentObservation, UiViewportExtentRequest,
};
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::application_authority_closure::fixed_host::FixedCertificationHostBinding;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_runtime::facade::entry::UiMountedAllocationMeasurementRequest;
use worth_ui_runtime::facade::host::{
    UiHostMeasurementAssumptionProfile, UiHostMeasurementNeed,
    UiHostMeasurementNormalizationContext,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFramePreparationDenial, UiMountedFrameRequest,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedAllocationCertificationExt, WorthUiMountedFrameExecutionCertificationExt,
    WorthUiMountedIdentityCertificationExt,
};

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::known_empty_surface_world::profile;

#[path = "filesystem_intent_world.rs"]
mod intent_world;
pub(super) use intent_world::{
    intent_world_operability_fact, launch_file_intent_world, launch_rust_command_intent_world,
    launch_rust_intent_world, prepare_rust_command_intent_application,
    prepare_rust_command_intent_application_with_policy, INTENT_WORLD_OPERABILITY_FACT,
};

#[derive(Clone, Copy)]
enum VisualWorldProfile {
    Canonical,
    Clipped,
}

pub(super) fn launch_clipped_world() -> worth_ui::facade::app::WorthUiActiveApplicationSession {
    let host = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    launch_world_with_host(
        VisualWorldProfile::Clipped,
        "phase-2-clipped-interaction",
        host,
        UiHostSurfacePresentationMode::RecordOnly,
    )
}

pub(super) fn launch_world() -> worth_ui::facade::app::WorthUiActiveApplicationSession {
    let host = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    launch_world_with_host(
        VisualWorldProfile::Canonical,
        "phase-3-visual-identity",
        host,
        UiHostSurfacePresentationMode::RecordOnly,
    )
}

fn launch_world_with_host<Host>(
    world_profile: VisualWorldProfile,
    responsibility: &str,
    host: Host,
    mode: UiHostSurfacePresentationMode,
) -> worth_ui::facade::app::WorthUiActiveApplicationSession
where
    Host: FixedCertificationHostBinding + Clone + 'static,
{
    let application = prepare_filesystem_application(world_profile, responsibility, host);
    let component_nodes = component_graph_nodes(&application);
    launch_mounted_components(application, component_nodes, mode)
}

fn prepare_filesystem_application<Host>(
    world_profile: VisualWorldProfile,
    responsibility: &str,
    host: Host,
) -> worth_ui::facade::app::WorthUiApp
where
    Host: FixedCertificationHostBinding + Clone + 'static,
{
    let scenario = FilesystemApplicationLifecycleScenario::new(responsibility);
    let workspace = FilesystemContractWorkspace::new(responsibility);
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::visual_identity_source_text(),
    );
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("production filesystem provider reads the visual identity source");
    let capabilities = match world_profile {
        VisualWorldProfile::Canonical => {
            scenario.visual_identity_capability_application(host.clone())
        }
        VisualWorldProfile::Clipped => {
            scenario.clipped_visual_identity_capability_application(host.clone())
        }
    };
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        snapshot,
        capabilities.capabilities(),
    );
    let application = match world_profile {
        VisualWorldProfile::Canonical => {
            scenario.prepare_visual_identity_application_with_host(submission, host)
        }
        VisualWorldProfile::Clipped => {
            scenario.prepare_clipped_visual_identity_application_with_host(submission, host)
        }
    };
    workspace.close();
    application
}

pub(crate) fn launch_mounted_components(
    application: worth_ui::facade::app::WorthUiApp,
    component_nodes: Vec<UiGraphNodeIdentity>,
    mode: UiHostSurfacePresentationMode,
) -> worth_ui::facade::app::WorthUiActiveApplicationSession {
    let mut session = application
        .launch()
        .expect("the real filesystem visual identity application launches");
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(surface, mode, profile(1))
        .unwrap();
    for graph_node in component_nodes {
        let handle = session.mounted_graph_node(graph_node).unwrap();
        session.mount_instance(handle, surface).unwrap();
    }
    session
}

pub(crate) fn component_graph_nodes(
    application: &worth_ui::facade::app::WorthUiApp,
) -> Vec<UiGraphNodeIdentity> {
    assert_eq!(application.graph().node_count(), 6);
    let component_nodes = (0..4)
        .map(|declaration_index| component_graph_node(application, declaration_index))
        .collect::<Vec<_>>();
    assert_structural_graph_world(application, &component_nodes);
    component_nodes
}

fn component_graph_node(
    application: &worth_ui::facade::app::WorthUiApp,
    declaration_index: usize,
) -> UiGraphNodeIdentity {
    let artifact = application
        .declaration_artifacts()
        .iter()
        .find(|artifact| {
            let source = artifact.provenance().source_provenance();
            source.module_path() == "app/main.wui"
                && source.declaration_index() == declaration_index
        })
        .unwrap_or_else(|| {
            panic!("component declaration {declaration_index} must survive real lowering")
        });
    assert_eq!(
        artifact.graph_handoff().unwrap().role(),
        UiDeclarationStructuralRole::Mosaic
    );
    let lookup = application
        .graph()
        .lookup()
        .declaration_instances(artifact.identity());
    assert_eq!(
        lookup.receipt().cost_class(),
        UiGraphLookupCostClass::IndexedSet
    );
    assert_eq!(lookup.value().len(), 1);
    lookup.value()[0]
}

fn assert_structural_graph_world(
    application: &worth_ui::facade::app::WorthUiApp,
    component_nodes: &[UiGraphNodeIdentity],
) {
    let source_surface = application
        .declaration_artifacts()
        .iter()
        .find(|artifact| {
            let source = artifact.provenance().source_provenance();
            source.module_path() == "app/main.wui" && source.declaration_index() == 4
        })
        .expect("the authored surface remains a declaration artifact");
    assert_eq!(
        application
            .graph()
            .lookup()
            .declaration_instances(source_surface.identity())
            .value()
            .len(),
        1
    );
    let bootstrap_nodes = application
        .declaration_artifacts()
        .iter()
        .filter(|artifact| {
            artifact.provenance().source_provenance().module_path() != "app/main.wui"
        })
        .map(|artifact| {
            application
                .graph()
                .lookup()
                .declaration_instances(artifact.identity())
                .value()
                .len()
        })
        .sum::<usize>();
    assert_eq!(bootstrap_nodes, 1);
    let unselected_nodes = application
        .graph()
        .node_identities()
        .filter(|node| !component_nodes.contains(node))
        .count();
    assert_eq!(
        unselected_nodes, 2,
        "the authored surface and bootstrap page remain real graph members"
    );
}

pub(super) fn establish_allocation(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    expected_receipts: usize,
) {
    let capability = session.host_measurement_capability();
    let assumptions = UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let input = UiMountedAllocationMeasurementRequest::new(
        UiMeasurementEvidenceFamily::ViewportExtent,
        UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
        UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    let receipt = session
        .establish_mounted_allocation_catalog(1, [input])
        .expect("host viewport measurement establishes production allocation");
    assert_eq!(receipt.committed().receipts().len(), expected_receipts);
}

pub(super) fn prepare_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> Result<
    worth_ui_runtime::facade::mounted::UiPreparedMountedFrame,
    UiMountedFramePreparationDenial,
> {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("no presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("the visual identity world admits ordinary execution"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
}
