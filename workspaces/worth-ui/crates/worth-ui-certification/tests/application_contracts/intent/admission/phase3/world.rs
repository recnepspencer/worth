use worth_ui::facade::app::WorthUiActiveApplicationSession;
use worth_ui::facade::intent::{
    UiAdmittedIntent, UiIntentAdmissionDecision, UiIntentDefinition, UiIntentOperabilityOutcome,
    UiIntentRouteResolution, UiIntentRouteSource,
};
use worth_ui::facade::interaction::{
    UiHostInteractionIngressOutcome, UiInteractionTransition, UiSemanticInteraction,
};
use worth_ui::facade::observation_report::{
    UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerIdentity, UiHostPressedPointerButtons,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedInstanceIdentity,
    UiPresentationDeadline, UiSurfaceBindingGeneration,
};
use worth_ui_test_support::{
    WorthUiMountedIdentityCertificationExt, WorthUiMountedInteractionLifecycleCertificationExt,
    WorthUiMountedPublicationCertificationExt,
};

use super::super::super::operability::{
    build_scoped, build_scoped_with_provider, build_scoped_with_provider_observation,
    OccupancyLayout, OperabilityFacts, PrimaryIntent,
};
use crate::filesystem_mounted_world::{component_graph_nodes, establish_allocation, prepare_frame};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_application_lifecycle::published_mounted_world::presented_epoch;

const TARGET_POINT: [i64; 2] = [10, 20];

mod observation;
mod replacement;
mod semantic_text_launch;

use observation::{pointer_button, position};

pub(in crate::intent) struct AdmissionWorld {
    pub(in crate::intent) session: WorthUiActiveApplicationSession,
    facts: OperabilityFacts,
    targets: Box<[AdmissionTarget]>,
    next_pointer: u64,
    next_sequence: u64,
    target_point: [i64; 2],
}

#[derive(Clone, Copy)]
struct AdmissionTarget {
    presentation: UiHostObservationPresentationBasis,
    mounted_instance: UiMountedInstanceIdentity,
}

impl AdmissionWorld {
    pub(in crate::intent) fn launch(target_count: usize) -> Self {
        assert!(target_count > 0, "an admission world needs one real target");
        let (application, facts) = build_scoped(OccupancyLayout::TargetRoute);
        Self::launch_application(application, facts, target_count)
    }

    pub(in crate::intent) fn launch_with_provider_observation(
        target_count: usize,
    ) -> (
        Self,
        worth_ui_certification::WorthUiCertificationProviderObservation,
    ) {
        assert!(target_count > 0, "an admission world needs one real target");
        let (application, facts, observation) =
            build_scoped_with_provider_observation(OccupancyLayout::TargetRoute);
        (
            Self::launch_application(application, facts, target_count),
            observation,
        )
    }

    pub(in crate::intent) fn launch_with_provider<P>(target_count: usize, provider: P) -> Self
    where
        P: worth_ui::facade::intent::UiIntentExecutionProvider<PrimaryIntent>,
    {
        assert!(target_count > 0, "an admission world needs one real target");
        let (application, facts) =
            build_scoped_with_provider(OccupancyLayout::TargetRoute, provider);
        Self::launch_application(application, facts, target_count)
    }

    pub(in crate::intent) fn launch_application(
        application: worth_ui::facade::app::WorthUiApp,
        facts: OperabilityFacts,
        target_count: usize,
    ) -> Self {
        Self::launch_application_with_routed_component(application, facts, target_count, 1)
    }

    pub(in crate::intent) fn launch_application_with_routed_component(
        application: worth_ui::facade::app::WorthUiApp,
        facts: OperabilityFacts,
        target_count: usize,
        routed_component_index: usize,
    ) -> Self {
        Self::launch_application_with_target(
            application,
            facts,
            target_count,
            routed_component_index,
            TARGET_POINT,
        )
    }

    pub(in crate::intent) fn launch_application_with_target(
        application: worth_ui::facade::app::WorthUiApp,
        facts: OperabilityFacts,
        target_count: usize,
        routed_component_index: usize,
        target_point: [i64; 2],
    ) -> Self {
        let nodes = component_graph_nodes(&application);
        assert!(routed_component_index < nodes.len());
        let mut session = application
            .launch()
            .expect("admission application launches");
        let mounted =
            mount_complete_pages(&mut session, &nodes, target_count, routed_component_index);
        establish_allocation(&mut session, 3);
        let prepared = prepare_frame(&mut session).expect("admission frame prepares");
        assert_eq!(prepared.surfaces().len(), target_count);
        let publication = match session.present_prepared_mounted_frame(
            prepared,
            UiPresentationDeadline::at_tick(1_000),
            0,
        ) {
            UiMountedFrameOutcome::Published(publication) => publication,
            _ => panic!("admission frame must publish"),
        };
        let targets = mounted
            .into_iter()
            .map(|(binding, mounted_instance)| AdmissionTarget {
                presentation: presentation(&session, publication.frame(), binding),
                mounted_instance,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            session,
            facts,
            targets,
            next_pointer: 1,
            next_sequence: 1,
            target_point,
        }
    }

    pub(in crate::intent) fn admit(
        &mut self,
        target: usize,
    ) -> UiIntentAdmissionDecision<PrimaryIntent> {
        let outcome = self.evaluate(target);
        self.session.admit_intent(
            UiIntentDefinition::<PrimaryIntent>::application_effect(),
            outcome,
        )
    }

    pub(in crate::intent) fn admit_exact(
        &mut self,
        target: usize,
    ) -> UiAdmittedIntent<PrimaryIntent> {
        self.admit_exact_for::<PrimaryIntent>(target)
    }

    pub(in crate::intent) fn admit_exact_for<I: worth_ui::facade::intent::UiIntent>(
        &mut self,
        target: usize,
    ) -> UiAdmittedIntent<I> {
        self.admit_exact_definition(target, UiIntentDefinition::<I>::application_effect())
    }

    pub(in crate::intent) fn admit_exact_definition<I, D>(
        &mut self,
        target: usize,
        definition: UiIntentDefinition<I, D>,
    ) -> UiAdmittedIntent<I>
    where
        I: worth_ui::facade::intent::UiIntent,
        D: worth_ui::facade::intent::UiIntentDefinitionDestination,
    {
        let outcome = self.evaluate(target);
        match self.session.admit_intent(definition, outcome) {
            UiIntentAdmissionDecision::Admitted(admitted) => admitted,
            UiIntentAdmissionDecision::ConfirmationRequired(_) => {
                panic!("confirmation-disabled admission cannot require confirmation")
            }
            UiIntentAdmissionDecision::Stopped(stop) => {
                panic!("current operable target must admit: {:?}", stop.reason())
            }
        }
    }

    pub(in crate::intent) fn evaluate(&mut self, target: usize) -> UiIntentOperabilityOutcome {
        let interaction = self.activation(target);
        let route = match self
            .session
            .resolve_intent_route(UiIntentRouteSource::mounted_interaction(interaction))
            .expect("admission target resolves its product route")
        {
            UiIntentRouteResolution::Product(route) => route,
            UiIntentRouteResolution::Confirmation(_) => {
                panic!("product target cannot resolve as confirmation")
            }
        };
        let payload = self
            .session
            .prepare_intent_payload(route)
            .expect("empty admission payload prepares");
        self.session.evaluate_intent_operability(payload)
    }

    pub(in crate::intent) fn set_policy(&mut self, admitted: bool) {
        self.session
            .update_intent_boolean_fact(&self.facts.policy, admitted)
            .expect("admission policy update is owner-issued");
    }

    pub(in crate::intent) fn motion_storm(&mut self, target: usize, count: usize) {
        for _ in 0..count {
            let pointer = self.take_pointer();
            let outcome = self.observe(
                target,
                UiHostObservationPayload::PointerMotion {
                    pointer: UiHostPointerIdentity::new(pointer),
                    capture_epoch: UiHostPointerCaptureEpoch::new(1),
                    pressed_buttons: UiHostPressedPointerButtons::NONE,
                    position: position(self.target_point),
                },
            );
            let UiHostInteractionIngressOutcome::Applied(receipt) = outcome else {
                panic!("motion observation reaches the production interaction owner")
            };
            assert!(receipt
                .transitions()
                .iter()
                .all(|transition| !matches!(transition, UiInteractionTransition::Semantic(_))));
        }
    }

    pub(in crate::intent) fn unmount(
        &mut self,
        target: usize,
    ) -> Result<(), worth_ui_runtime::facade::mounted::UiMountedIdentityDenial> {
        self.session
            .unmount_instance_with_interaction_receipt(self.targets[target].mounted_instance)
            .map(|_| ())
    }

    fn activation(&mut self, target: usize) -> UiSemanticInteraction {
        let pointer = self.take_pointer();
        let target_point = self.target_point;
        let _ = self.observe(
            target,
            pointer_button(
                pointer,
                UiHostPointerButtonTransition::Pressed,
                target_point,
            ),
        );
        let released = self.observe(
            target,
            pointer_button(
                pointer,
                UiHostPointerButtonTransition::Released,
                target_point,
            ),
        );
        let UiHostInteractionIngressOutcome::Applied(receipt) = released else {
            panic!("release reaches the production interaction owner: {released:?}")
        };
        let semantic = receipt
            .into_transitions()
            .into_vec()
            .into_iter()
            .find_map(|transition| match transition {
                UiInteractionTransition::Semantic(interaction) => Some(interaction),
                _ => None,
            })
            .expect("complete press/release mints one semantic activation");
        assert_eq!(
            semantic.target().mounted_instance(),
            self.targets[target].mounted_instance,
            "surface-bound observation must target the requested incarnation"
        );
        semantic
    }

    fn take_pointer(&mut self) -> u64 {
        let pointer = self.next_pointer;
        self.next_pointer += 1;
        pointer
    }
}

fn mount_complete_pages(
    session: &mut WorthUiActiveApplicationSession,
    nodes: &[worth_ui::facade::graph::UiGraphNodeIdentity],
    target_count: usize,
    routed_component_index: usize,
) -> Vec<(UiSurfaceBindingGeneration, UiMountedInstanceIdentity)> {
    (0..target_count)
        .map(|index| {
            let surface = session.create_semantic_surface().unwrap();
            let binding = session
                .register_host_surface(
                    surface,
                    UiHostSurfacePresentationMode::RecordOnly,
                    profile(index as u64 + 1),
                )
                .unwrap()
                .binding_generation();
            let mut routed_instance = None;
            for (node_index, graph_node) in nodes.iter().copied().enumerate() {
                let handle = session.mounted_graph_node(graph_node).unwrap();
                let mounted = session.mount_instance(handle, surface).unwrap();
                if node_index == routed_component_index {
                    routed_instance = Some(mounted);
                }
            }
            let routed_instance =
                routed_instance.expect("the complete page includes its routed hit-only component");
            (binding, routed_instance)
        })
        .collect()
}

fn presentation(
    session: &WorthUiActiveApplicationSession,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    binding: UiSurfaceBindingGeneration,
) -> UiHostObservationPresentationBasis {
    let host_surface = session
        .inspect_mounted_identity()
        .surface_bindings()
        .iter()
        .find(|candidate| candidate.binding_generation() == binding)
        .expect("the target binding remains current")
        .host_surface_identity();
    UiHostObservationPresentationBasis::new(
        host_surface,
        frame,
        binding,
        presented_epoch(session, frame, binding),
    )
}
