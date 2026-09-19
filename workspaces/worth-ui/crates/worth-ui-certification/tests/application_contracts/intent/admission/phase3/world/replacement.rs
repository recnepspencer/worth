use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui::facade::source::{
    WorthUiSourceIngressExt, WorthUiSourceProvider, WorthUiWatcherEvent,
};

use super::AdmissionWorld;

impl AdmissionWorld {
    pub(in crate::intent) fn rebind_application(
        &mut self,
    ) -> (
        worth_ui::facade::observation::UiObservationResourceRetirementReport,
        worth_ui_inspection::UiIntentEvidenceRetirementReport,
    ) {
        let input = self.replacement_input.clone();
        self.rebind_application_with_input(input)
    }

    pub(in crate::intent) fn rebind_application_with_input(
        &mut self,
        input: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
    ) -> (
        worth_ui::facade::observation::UiObservationResourceRetirementReport,
        worth_ui_inspection::UiIntentEvidenceRetirementReport,
    ) {
        let admitted = self
            .begin_replacement_observation_turn_with_input("phase-4-execution-replacement", input)
            .seal()
            .expect("replacement observation turn seals");
        let changed = match self.session.classify_observations(admitted).unwrap() {
            UiChangeClassificationOutcome::Changed(changed) => changed,
            _ => panic!("changed intent concurrency is executable replacement meaning"),
        };
        let lifecycle = self
            .session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        let plan = self
            .session
            .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
            .unwrap();
        let prepared = self
            .session
            .prepare_rebind(plan, UiRebindExecutionRequest::new(30))
            .expect("application replacement prepares through the native owner");
        let UiRebindOutcome::Published(receipt) = prepared.execute(30) else {
            panic!("application replacement must publish")
        };
        let application = receipt
            .application_publication()
            .expect("changed application replacement owns one cutover receipt");
        (
            application.observation_resources(),
            application.intent_evidence(),
        )
    }

    pub(in crate::intent) fn begin_replacement_observation_turn(
        &mut self,
    ) -> worth_ui::facade::observation::UiObservationTurn<'_> {
        let input = self.replacement_input.clone();
        self.begin_replacement_observation_turn_with_input("phase-4-execution-replacement", input)
    }

    pub(in crate::intent) fn begin_replacement_observation_turn_for_provider(
        &mut self,
        provider: &'static str,
    ) -> worth_ui::facade::observation::UiObservationTurn<'_> {
        let input = self.replacement_input.clone();
        self.begin_replacement_observation_turn_with_input(provider, input)
    }

    fn begin_replacement_observation_turn_with_input(
        &mut self,
        provider_name: &'static str,
        input: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
    ) -> worth_ui::facade::observation::UiObservationTurn<'_> {
        let provider =
            WorthUiSourceProvider::rust_authored(provider_name).with_rust_authored_input(input);
        let mut ingress = self.session.source_event_ingress(provider).start();
        let settled = ingress
            .ingest([WorthUiWatcherEvent::provider_revision(provider_name)])
            .expect("replacement source settles through production ingress");
        let submission = settled
            .attempt_candidate_for_certification(self.session.capabilities())
            .expect("replacement source lowers through the production compiler");
        let mut turn = self.session.begin_observation_turn().unwrap();
        turn.admit_source(submission).unwrap();
        turn
    }
}
