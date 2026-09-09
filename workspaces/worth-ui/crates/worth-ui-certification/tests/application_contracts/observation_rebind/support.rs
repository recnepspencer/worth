use worth_ui::facade::app::WorthUiActiveApplicationSession;
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiPreparedRebind, UiRebindExecutionPolicy, UiRebindExecutionRequest,
};
use worth_ui::facade::source::{WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameRequest, UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiFrameworkTurnCertificationExt, WorthUiMountedFrameExecutionCertificationExt,
    WorthUiMountedIdentityCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;
use crate::mounted_application_lifecycle::known_empty_surface_world::{first_node, profile};
use crate::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

pub(crate) struct RebindExecutionWorld {
    scenario: FilesystemApplicationLifecycleScenario,
    workspace: FilesystemContractWorkspace,
    watcher: WorthUiFilesystemSourceWatcher,
    pub(crate) host: ScriptedPresentationHost,
    pub(crate) session: WorthUiActiveApplicationSession,
}

impl RebindExecutionWorld {
    pub(crate) fn new(label: &str) -> Self {
        let scenario = FilesystemApplicationLifecycleScenario::new(label);
        let workspace = FilesystemContractWorkspace::new(label);
        workspace.write(
            "app/main.wui",
            &FilesystemApplicationLifecycleScenario::dual_generation_scope_initial_source_text(),
        );
        let provider = WorthUiFilesystemSourceProvider::new(workspace.root());
        let mut watcher = WorthUiFilesystemSourceWatcher::start(provider)
            .expect("production filesystem watcher starts");
        let capabilities = scenario.capability_application();
        let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
            watcher
                .take_initial_snapshot()
                .expect("watcher owns the initial settled source"),
            capabilities.capabilities(),
        );
        let host = ScriptedPresentationHost::default();
        host.set_visual_capture_capability(
            worth_ui_host_contract::UiHostCaptureCapability::Pixels {
                maximum_bytes: 1_024,
                exact_presentation_epoch: true,
            },
        );
        let mut session = scenario
            .prepare_application_with_host(submission, host.clone())
            .launch()
            .expect("filesystem-authored application launches");
        mount_one_surface(&mut session);
        host.push_presented();
        publish_predecessor(&mut session);
        Self {
            scenario,
            workspace,
            watcher,
            host,
            session,
        }
    }

    pub(crate) fn prepare_changed(&mut self) -> UiPreparedRebind<'_> {
        let plan = self.changed_plan();
        self.session
            .prepare_rebind(plan, UiRebindExecutionRequest::new(1))
            .expect("current changed plan prepares")
    }

    pub(super) fn changed_plan(&mut self) -> worth_ui::facade::rebind::UiRebindPlan {
        self.workspace.write_atomic(
            "app/main.wui",
            &FilesystemApplicationLifecycleScenario::dual_generation_scope_candidate_source_text(),
        );
        let snapshot = self
            .watcher
            .settle(Duration::from_secs(5))
            .expect("candidate filesystem source settles");
        let candidate = snapshot
            .attempt_candidate_for_certification(self.session.capabilities())
            .expect("candidate filesystem source lowers");
        let mut turn = self.session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        let admitted = turn.seal().unwrap();
        let changed = match self.session.classify_observations(admitted).unwrap() {
            UiChangeClassificationOutcome::Changed(changed) => changed,
            _ => panic!("candidate source must change application semantics"),
        };
        let lifecycle = self
            .session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        self.session
            .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
            .expect("changed lifecycle compiles to one exact plan")
    }

    pub(crate) fn evidence_only_plan(&mut self) -> worth_ui::facade::rebind::UiRebindPlan {
        let source = format!(
            "{}\n",
            FilesystemApplicationLifecycleScenario::dual_generation_scope_initial_source_text()
        );
        self.workspace.write_atomic("app/main.wui", &source);
        let snapshot = self
            .watcher
            .settle(Duration::from_secs(5))
            .expect("evidence-only filesystem source settles");
        let candidate = snapshot
            .attempt_candidate_for_certification(self.session.capabilities())
            .expect("evidence-only filesystem source lowers");
        let mut turn = self.session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        let admitted = turn.seal().unwrap();
        let evidence = match self.session.classify_observations(admitted).unwrap() {
            UiChangeClassificationOutcome::EvidenceOnly(evidence) => evidence,
            _ => panic!("provenance-only source edit must preserve semantics"),
        };
        self.session
            .compile_preservation_rebind(evidence, UiRebindExecutionPolicy::ordinary())
            .expect("evidence-only succession compiles to a preservation plan")
    }

    pub(crate) fn close(self) {
        let shutdown = self.session.shutdown();
        assert!(shutdown.rebind().is_empty());
        assert!(shutdown.mounted_presentation().is_empty());
        self.watcher
            .shutdown()
            .expect("production filesystem watcher shuts down");
        self.workspace.close();
        drop(self.scenario);
    }
}

fn mount_one_surface(session: &mut WorthUiActiveApplicationSession) {
    let node = first_node(session);
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    session.mount_instance(node, surface).unwrap();
}

fn publish_predecessor(session: &mut WorthUiActiveApplicationSession) {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let prepared = session
        .execute_framework_turn(|_| {})
        .expect("initial framework turn is available")
        .into_execution()
        .unwrap_or_else(|_| panic!("initial framework turn produces execution"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
        .expect("initial mounted frame prepares");
    let outcome =
        session.present_prepared_mounted_frame(prepared, UiPresentationDeadline::at_tick(10), 0);
    let _ = match outcome {
        worth_ui_runtime::facade::mounted::UiMountedFrameOutcome::Published(receipt) => receipt,
        _ => panic!("scripted initial frame must publish"),
    };
}
use std::time::Duration;
