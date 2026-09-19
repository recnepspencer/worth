use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::application_authority_closure::fixed_host::FixedCertificationHostBinding;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;

pub(crate) fn active_session() -> worth_ui::facade::app::WorthUiActiveApplicationSession {
    mounted_application("mounted-identity")
        .launch()
        .expect("runtime should launch from the real filesystem-authored world")
}

fn mounted_application(label: &str) -> worth_ui::facade::app::WorthUiApp {
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let submission = mounted_submission(
        label,
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
        &scenario,
    );
    scenario.prepare_application_with_host(
        submission,
        worth_ui_host_headless::WorthUiHeadlessRecorder::default(),
    )
}

pub(crate) fn mounted_application_with_host<Host>(
    label: &str,
    host: Host,
) -> worth_ui::facade::app::WorthUiApp
where
    Host: FixedCertificationHostBinding + 'static,
{
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let submission = mounted_submission(
        label,
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
        &scenario,
    );
    scenario.prepare_application_with_host(submission, host)
}

pub(crate) fn mounted_application_with_host_and_visual_policy<Host>(
    label: &str,
    host: Host,
    policy: worth_ui::facade::inspection::UiVisualInspectionPolicy,
) -> worth_ui::facade::app::WorthUiApp
where
    Host: FixedCertificationHostBinding + 'static,
{
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let submission = mounted_submission(
        label,
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
        &scenario,
    );
    scenario.prepare_application_with_host_and_visual_policy(submission, host, policy)
}

pub(crate) fn mounted_application_with_host_and_retention_budget<Host>(
    label: &str,
    host: Host,
    retention_budget: worth_ui_runtime::facade::mounted::UiMountedFrameRetentionBudget,
) -> worth_ui::facade::app::WorthUiApp
where
    Host: FixedCertificationHostBinding + 'static,
{
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let submission = mounted_submission(
        label,
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
        &scenario,
    );
    scenario.prepare_application_with_host_and_retention_budget(submission, host, retention_budget)
}

pub(crate) fn mounted_application_with_host_and_capacities<Host>(
    label: &str,
    host: Host,
    retention_budget: worth_ui_runtime::facade::mounted::UiMountedFrameRetentionBudget,
    observation_capacity: worth_ui::facade::observation_report::UiHostObservationCapacity,
) -> worth_ui::facade::app::WorthUiApp
where
    Host: FixedCertificationHostBinding + 'static,
{
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let submission = mounted_submission(
        label,
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
        &scenario,
    );
    scenario.prepare_application_with_host_and_capacities(
        submission,
        host,
        retention_budget,
        observation_capacity,
    )
}

fn mounted_submission(
    label: &str,
    source: &str,
    scenario: &FilesystemApplicationLifecycleScenario,
) -> worth_ui::facade::source::WorthUiWatchedCandidateSubmission {
    let workspace = FilesystemContractWorkspace::new(label);
    workspace.write("app/main.wui", source);
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("production filesystem acquisition should read actual .wui bytes");
    workspace.close();
    let capabilities = scenario.capability_application();
    FilesystemApplicationLifecycleScenario::lower_snapshot(snapshot, capabilities.capabilities())
}

pub(crate) fn registered_surface(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    surface
}

pub(crate) fn first_node(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiMountedGraphNodeHandle {
    let graph = session.graph();
    let node = graph
        .node_identities()
        .find(|node| {
            graph.lookup().graph_node(*node).is_some_and(|record| {
                record
                    .value()
                    .declaration_identity()
                    .authored_semantic_name()
                    == "component:workspace.component.authority_current"
            })
        })
        .expect("the mounted fixture selects its declared current component");
    session.mounted_graph_node(node).unwrap()
}

pub(crate) fn profile(epoch: u64) -> UiSurfaceBindingProfile {
    UiSurfaceBindingProfile::new(
        1_000,
        UiSurfaceBindingCoordinatePosture::LogicalPoints,
        epoch,
    )
    .unwrap()
}
