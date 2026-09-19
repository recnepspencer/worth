use super::super::super::operability::{OperabilityFacts, PrimaryIntent};
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
use worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity;
use worth_ui_test_support::WorthUiMountedInteractionLifecycleCertificationExt;

const TARGET_POINT: [i64; 2] = [10, 20];

mod launch;
mod observation;
mod replacement;

use observation::{pointer_button, position};

pub(in crate::intent) struct AdmissionWorld {
    pub(in crate::intent) session: WorthUiActiveApplicationSession,
    facts: OperabilityFacts,
    targets: Box<[AdmissionTarget]>,
    next_pointer: u64,
    next_sequence: u64,
    target_point: [i64; 2],
    replacement_input: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
}

#[derive(Clone, Copy)]
struct AdmissionTarget {
    presentation: UiHostObservationPresentationBasis,
    mounted_instance: UiMountedInstanceIdentity,
}

impl AdmissionWorld {
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
