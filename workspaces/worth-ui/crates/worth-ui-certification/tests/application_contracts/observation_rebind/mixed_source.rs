use worth_ui::facade::measurement_exchange::{
    UiHostMeasurementOutcome, UiMeasurementEvidenceFamily, UiPortalAnchorRectRequest,
    UiViewportExtentRequest, WorthUiHostMeasurementSessionExt,
};
use worth_ui::facade::observation::{UiChangeClassificationOutcome, UiObservationFamily};
use worth_ui::facade::observation_report::{
    UiHostObservationLoss, UiHostObservationPayload, UiHostObservationReportOutcome,
    WorthUiHostObservationSessionExt,
};
use worth_ui::facade::rebind::UiProducedFactFamily;
use worth_ui::facade::source::{
    WorthUiSourceEventIngress, WorthUiSourceProvider, WorthUiWatcherEvent,
};
use worth_ui_dsl::{
    UiDslPostureToken, UiDslSemanticFamily, UiDslSemanticKey, UiDslStructuralToken,
    WorthUiArtifactInputBodyAtom, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule, WorthUiSemanticArtifactDeclaration,
};
use worth_ui_query_binding::{
    WorthUiOperationLiveRefreshOutcome, WorthUiOperationLiveRefreshRequest,
    WorthUiQueryWorkspaceExt,
};
use worth_ui_runtime::facade::entry::UiMountedAllocationMeasurementRequest;
use worth_ui_runtime::facade::host::{
    UiHostMeasurementAssumptionProfile, UiHostMeasurementNeed,
    UiHostMeasurementNormalizationContext, UiPortalAnchorCoordinateSpacePosture,
};
use worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode;
use worth_ui_test_support::{
    UiDeclaredMeasurementBasisSource, WorthUiActiveSessionCertificationExt,
    WorthUiMountedAllocationCertificationExt, WorthUiMountedIdentityCertificationExt,
};

use crate::query_replacement_lifecycle::{
    query_patch::update_measurement,
    scenario::{
        application, installed_workspace_with_measurement_authority, submission, FIRST_VIEW,
        NEXT_COMPONENT, SECOND_VIEW,
    },
    support::{admit_active_resource, close_retirement},
};
use crate::{
    host_measurement_fixture::{begin_viewport, measurement_host, viewport_observation},
    host_observation_fixture::{batch, report, source},
    mounted_application_lifecycle::known_empty_surface_world::profile,
    mounted_application_lifecycle::published_mounted_world::publish,
};

const RUNTIME_STATE_COMPONENT: &str = "workspace.component.authority_candidate";
const RUNTIME_STATE_REGION: &str = "workspace.region.authority_primary";
const RUNTIME_STATE_SIZING: &str = "workspace.sizing.authority_primary";

#[test]
fn real_source_and_query_consequences_share_one_canonically_ordered_turn() {
    let (mut workspace, measurement) =
        installed_workspace_with_measurement_authority("phase-312-tt05-mixed");
    let installed = workspace
        .worth_ui()
        .expect("Worth UI Query domain is installed");
    let first = installed.live_measurement_view(FIRST_VIEW).unwrap();
    let second = installed.live_measurement_view(SECOND_VIEW).unwrap();
    let mut session = application(first.clone(), second, &mut workspace)
        .launch()
        .expect("real Query-backed application launches");
    let reference = admit_active_resource(&mut session, &first, &mut workspace);
    update_measurement(&measurement, &mut workspace);
    let query = match session
        .refresh_query_change(WorthUiOperationLiveRefreshRequest::new(
            &reference,
            &mut workspace,
        ))
        .expect("Query owner progresses the exact installed resource")
    {
        WorthUiOperationLiveRefreshOutcome::Applied(consequence) => consequence,
        WorthUiOperationLiveRefreshOutcome::NoSemanticDelivery => {
            panic!("changed Query value must issue one UI consequence")
        }
    };
    let source = submission(
        "phase-312-tt05-source",
        NEXT_COMPONENT,
        &[FIRST_VIEW],
        session.capabilities(),
    );
    let generation = session.generation_identity().clone();

    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_query(query).unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    assert_eq!(
        admitted.summary().families(),
        &[
            UiObservationFamily::AuthoredSource,
            UiObservationFamily::Query
        ]
    );
    assert_eq!(admitted.observations()[0].owner_order(), 1);
    assert_eq!(admitted.observations()[1].query_change_order(), Some(1));
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("source and Query meaning must classify as changed"),
    };
    assert_eq!(
        changed
            .facts()
            .iter()
            .map(|fact| fact.family())
            .collect::<Vec<_>>(),
        [
            UiProducedFactFamily::AuthoredSource,
            UiProducedFactFamily::AuthoredSource,
            UiProducedFactFamily::Query,
        ]
    );
    assert_eq!(session.generation_identity(), &generation);
    close_retirement(
        session.shutdown().into_operation_live_retirement(),
        &mut workspace,
    );
}

#[test]
fn source_host_measurement_and_committed_runtime_state_share_owner_coordinates() {
    let scenario = worth_ui_certification::scenario::filesystem_application_lifecycle::
        FilesystemApplicationLifecycleScenario::new("phase-312-tt05-mechanical");
    let capabilities = scenario.capability_application();
    let submission = runtime_state_submission(
        "phase-312-tt05-runtime-initial",
        capabilities.capabilities(),
    );
    let host = measurement_host();
    let mut session = scenario
        .prepare_application_with_host(submission, host.clone())
        .launch()
        .expect("portal-authored application launches");
    assert!(
        session
            .measurement_basis_sources()
            .contains(&UiDeclaredMeasurementBasisSource::PortalAnchor),
        "authored portal posture must reach the active declaration authority"
    );
    let (binding, instance) = mount_all_runtime_state_nodes(&mut session);
    establish_runtime_state_catalog(&mut session);
    let _predecessor = publish(&mut session, &host, instance);
    let current = publish(&mut session, &host, instance);
    let source_candidate =
        runtime_state_submission("phase-312-tt05-runtime-turn", session.capabilities());
    let host_observation = match session.validate_host_observation_batch(batch(
        source(&session, binding, &current),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            UiHostObservationPayload::Viewport {
                width_subpixels: 80_000,
                height_subpixels: 60_000,
            },
            &current,
        )],
    )) {
        UiHostObservationReportOutcome::Validated(batch) => batch,
        other => panic!("current mounted viewport report validates: {other:?}"),
    };
    let request = begin_viewport(&mut session, Some(binding), 100, 0);
    let measurement =
        match session.complete_host_measurement(viewport_observation(&request, 800.0, 600.0), 1) {
            UiHostMeasurementOutcome::Completed(result) => result,
            other => panic!("solicited measurement completes: {other:?}"),
        };

    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_measurement(measurement).unwrap();
    turn.admit_host(host_observation).unwrap();
    turn.admit_source(source_candidate).unwrap();
    let runtime = turn.admit_committed_runtime_state().unwrap();
    let runtime_families = runtime
        .admitted()
        .iter()
        .map(|receipt| receipt.family())
        .collect::<Vec<_>>();
    let admitted = turn.seal().unwrap();
    assert_eq!(
        &admitted.summary().families()[..3],
        &[
            UiObservationFamily::AuthoredSource,
            UiObservationFamily::HostViewport,
            UiObservationFamily::Measurement,
        ]
    );
    assert_eq!(
        &admitted.summary().families()[3..],
        runtime_families.as_slice()
    );
    assert!(
        !runtime_families.is_empty(),
        "the real mounted world must contribute committed runtime-state evidence"
    );
    assert!(runtime_families.iter().all(|family| matches!(
        family,
        UiObservationFamily::CommittedScrollExtent | UiObservationFamily::CommittedPortalAnchor
    )));
    drop(admitted);
    let _ = session.shutdown();
    drop(capabilities);
}

fn runtime_state_submission(
    provider_id: &str,
    capabilities: &worth_ui::facade::diagnostics::CapabilitySnapshot,
) -> worth_ui::facade::source::WorthUiWatchedCandidateSubmission {
    let component_body = vec![
        identifier("region"),
        identifier(RUNTIME_STATE_REGION),
        WorthUiArtifactInputBodyAtom::LeftBrace,
        identifier("sizing"),
        identifier(RUNTIME_STATE_SIZING),
        WorthUiArtifactInputBodyAtom::Semicolon,
        WorthUiArtifactInputBodyAtom::RightBrace,
    ];
    let portal = WorthUiSemanticArtifactDeclaration::new(
        UiDslSemanticKey::new("workspace.control.tt05_portal"),
        UiDslSemanticFamily::Control,
    )
    .with_structural_token(UiDslStructuralToken::new("control:tt05-portal"))
    .with_structural_token(UiDslStructuralToken::new("operator:portal-anchor"))
    .with_posture_token(UiDslPostureToken::new("measurement:portal-anchored"));
    let module = WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_component_body_atoms(RUNTIME_STATE_COMPONENT, component_body)
        .with_semantic_declaration(portal);
    let provider = WorthUiSourceProvider::rust_authored(provider_id)
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]));
    WorthUiSourceEventIngress::new(provider)
        .start()
        .ingest([WorthUiWatcherEvent::provider_revision(provider_id)])
        .unwrap()
        .attempt_candidate_for_certification(capabilities)
        .unwrap()
}

fn mount_all_runtime_state_nodes(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> (
    worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
) {
    let surface = session.create_semantic_surface().unwrap();
    let _registered = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let mut first = None;
    for identity in session.graph().node_identities().collect::<Vec<_>>() {
        let node = session.mounted_graph_node(identity).unwrap();
        let mounted = session.mount_instance(node, surface).unwrap();
        first.get_or_insert(mounted);
    }
    let binding = session.inspect_mounted_identity().surface_bindings()[0].binding_generation();
    (
        binding,
        first.expect("portal-authored graph has mounted nodes"),
    )
}

fn establish_runtime_state_catalog(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let capability = session.host_measurement_capability();
    let assumptions = UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let viewport = UiMountedAllocationMeasurementRequest::new(
        UiMeasurementEvidenceFamily::ViewportExtent,
        UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
        UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    let portal = UiMountedAllocationMeasurementRequest::new(
        UiMeasurementEvidenceFamily::PortalAnchorRect,
        UiHostMeasurementNeed::PortalAnchorRect(UiPortalAnchorRectRequest::new(1)),
        UiHostMeasurementNormalizationContext::portal_anchor_logical_exact_in(
            UiPortalAnchorCoordinateSpacePosture::PortalLayer,
            assumptions,
        ),
    );
    let receipt = session
        .establish_mounted_allocation_catalog(1, [viewport, portal])
        .expect("real host evidence establishes scroll and portal allocation truth");
    let committed_sources = receipt.committed_basis_sources();
    let committed = receipt.committed();
    assert!(!committed.receipts().is_empty());
    assert!(
        committed_sources.contains(&Some(
            UiDeclaredMeasurementBasisSource::PortalAnchor
        )),
        "portal-authored allocation must retain its typed portal basis source: {committed_sources:?}"
    );
    assert!(
        committed
            .receipts()
            .iter()
            .any(|receipt| receipt
                .geometry_evidence()
                .portal_anchor_observation()
                .is_some()),
        "portal-authored allocation must commit portal anchor geometry; committed sources: {committed_sources:?}"
    );
}

fn identifier(value: &str) -> WorthUiArtifactInputBodyAtom {
    WorthUiArtifactInputBodyAtom::Identifier(value.to_owned())
}
