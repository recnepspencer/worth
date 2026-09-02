use worth_ui::facade::app::{WorthUiActiveApplicationSession, WorthUiApp};
use worth_ui::facade::intent::{
    UiAdmittedIntent, UiIntentAdmissionDecision, UiIntentDefinition, UiIntentOperabilityOutcome,
    UiIntentRouteResolution, UiIntentRouteSource,
};
use worth_ui::facade::interaction::{
    UiHostInteractionIngressOutcome, UiInteractionTransition, UiSemanticInteraction,
};
use worth_ui::facade::observation_report::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostObservationReport,
    UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
    UiHostPointerButton, UiHostPointerButtonTransition, UiHostPointerCaptureEpoch,
    UiHostPointerDeviceKind, UiHostPointerIdentity, UiHostProtocolContract,
    UiHostProtocolNegotiation, UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedInstanceIdentity,
    UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiMountedIdentityCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::PrimaryIntent;
use crate::filesystem_mounted_world::{establish_allocation, prepare_frame};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_application_lifecycle::published_mounted_world::presented_epoch;

const TARGET_POINT: [i64; 2] = [10, 20];

pub(in crate::intent) struct MountedRouteScaleWorld {
    pub(in crate::intent) session: WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    target: UiMountedInstanceIdentity,
    next_sequence: u64,
}

pub(in crate::intent) fn last_route_graph_node(
    application: &WorthUiApp,
    route_count: usize,
) -> worth_ui::facade::graph::UiGraphNodeIdentity {
    let artifacts = route_artifacts(application, route_count);
    let (_, artifact) = artifacts
        .last()
        .expect("route-scale world contains one last routed control");
    graph_node_for_artifact(application, artifact)
}

impl MountedRouteScaleWorld {
    pub(in crate::intent) fn launch(application: WorthUiApp, route_count: usize) -> Self {
        assert_eq!(
            route_count, 1,
            "mounted route integration is the one-route IA-10 boundary"
        );
        let graph_nodes = route_graph_nodes(&application, route_count);
        let mut session = application
            .launch()
            .expect("route-scale application launches");
        let (binding, target) = mount_complete_route_page(&mut session, &graph_nodes);
        establish_allocation(&mut session, route_count);
        let prepared = prepare_frame(&mut session).expect("route-scale frame prepares");
        let publication = match session.present_prepared_mounted_frame(
            prepared,
            UiPresentationDeadline::at_tick(1_000),
            0,
        ) {
            UiMountedFrameOutcome::Published(publication) => publication,
            _ => panic!("route-scale frame must publish"),
        };
        let presentation = UiHostObservationPresentationBasis::new(
            session.inspect_mounted_identity().surface_bindings()[0].host_surface_identity(),
            publication.frame(),
            binding,
            presented_epoch(&session, publication.frame(), binding),
        );
        Self {
            session,
            presentation,
            target,
            next_sequence: 1,
        }
    }

    pub(in crate::intent) fn admit(&mut self) -> UiAdmittedIntent<PrimaryIntent> {
        let interaction = self.activation();
        let route = match self
            .session
            .resolve_intent_route(UiIntentRouteSource::mounted_interaction(interaction))
            .expect("selected route resolves")
        {
            UiIntentRouteResolution::Product(route) => route,
            UiIntentRouteResolution::Confirmation(_) => {
                panic!("route-scale target cannot resolve as confirmation")
            }
        };
        let payload = self
            .session
            .prepare_intent_payload(route)
            .expect("route-scale payload prepares");
        let outcome = self.session.evaluate_intent_operability(payload);
        let UiIntentOperabilityOutcome::Operable(_) = &outcome else {
            panic!("selected route remains operable")
        };
        match self.session.admit_intent(
            UiIntentDefinition::<PrimaryIntent>::application_effect(),
            outcome,
        ) {
            UiIntentAdmissionDecision::Admitted(admitted) => admitted,
            UiIntentAdmissionDecision::ConfirmationRequired(_) => {
                panic!("route-scale admission cannot require confirmation")
            }
            UiIntentAdmissionDecision::Stopped(stop) => {
                panic!("route-scale admission stopped: {:?}", stop.reason())
            }
        }
    }

    fn activation(&mut self) -> UiSemanticInteraction {
        let _ = self.observe(UiHostPointerButtonTransition::Pressed);
        let released = self.observe(UiHostPointerButtonTransition::Released);
        let UiHostInteractionIngressOutcome::Applied(receipt) = released else {
            panic!("route-scale release reaches the interaction owner")
        };
        let interaction = receipt
            .into_transitions()
            .into_vec()
            .into_iter()
            .find_map(|transition| match transition {
                UiInteractionTransition::Semantic(interaction) => Some(interaction),
                _ => None,
            })
            .expect("route-scale press/release mints one activation");
        assert_eq!(interaction.target().mounted_instance(), self.target);
        interaction
    }

    fn observe(
        &mut self,
        transition: UiHostPointerButtonTransition,
    ) -> UiHostInteractionIngressOutcome {
        let sequence = UiHostObservationSequence::new(self.next_sequence);
        self.next_sequence += 1;
        let report = UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(1),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                button: UiHostPointerButton::Primary,
                transition,
                position: target_position(),
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .expect("route-scale pointer reports carry an explicit device kind");
        let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
            protocol: protocol(),
            host_session: self.session.host_session_identity().as_u64(),
            presentation: self.presentation,
            sequences: UiHostObservationSequenceRange::new(sequence, sequence),
            loss: UiHostObservationLoss::Complete,
            reports: vec![report],
        })
        .expect("route-scale observation batch is structurally valid");
        self.session.admit_host_interaction_batch(batch)
    }
}

fn route_graph_nodes(
    application: &WorthUiApp,
    route_count: usize,
) -> Vec<worth_ui::facade::graph::UiGraphNodeIdentity> {
    route_artifacts(application, route_count)
        .into_iter()
        .map(|(_, artifact)| graph_node_for_artifact(application, artifact))
        .collect()
}

fn route_artifacts(
    application: &WorthUiApp,
    route_count: usize,
) -> Vec<(usize, &worth_ui::facade::declaration::UiDeclarationArtifact)> {
    let mut artifacts = application
        .declaration_artifacts()
        .iter()
        .filter_map(|artifact| {
            let source = artifact.provenance().source_provenance();
            (source.module_path() == "app/main.wui"
                && artifact
                    .identity()
                    .authored_semantic_name()
                    .starts_with("component:"))
            .then_some((source.declaration_index(), artifact))
        })
        .collect::<Vec<_>>();
    artifacts.sort_by_key(|(index, _)| *index);
    assert_eq!(artifacts.len(), route_count);
    assert!(artifacts.windows(2).all(|pair| pair[0].0 < pair[1].0));
    artifacts
}

fn graph_node_for_artifact(
    application: &WorthUiApp,
    artifact: &worth_ui::facade::declaration::UiDeclarationArtifact,
) -> worth_ui::facade::graph::UiGraphNodeIdentity {
    let lookup = application
        .graph()
        .lookup()
        .declaration_instances(artifact.identity());
    assert_eq!(lookup.value().len(), 1);
    lookup.value()[0]
}

fn mount_complete_route_page(
    session: &mut WorthUiActiveApplicationSession,
    graph_nodes: &[worth_ui::facade::graph::UiGraphNodeIdentity],
) -> (
    worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    UiMountedInstanceIdentity,
) {
    graph_nodes
        .iter()
        .copied()
        .enumerate()
        .map(|(index, graph_node)| {
            let surface = session.create_semantic_surface().unwrap();
            let binding = session
                .register_host_surface(
                    surface,
                    UiHostSurfacePresentationMode::RecordOnly,
                    profile(index as u64 + 1),
                )
                .unwrap()
                .binding_generation();
            let handle = session.mounted_graph_node(graph_node).unwrap();
            let mounted = session.mount_instance(handle, surface).unwrap();
            (binding, mounted)
        })
        .last()
        .expect("route-scale world contains at least one routed control")
}

fn target_position() -> UiHostSurfacePosition {
    UiHostSurfacePosition::viewport_logical(
        TARGET_POINT[0] * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        TARGET_POINT[1] * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    )
}

fn protocol() -> worth_ui::facade::observation_report::UiHostProtocolAgreement {
    match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    }
}
