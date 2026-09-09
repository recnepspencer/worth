use worth_ui::facade::intent::{
    UiIntentConsequenceContract, UiIntentConsequenceHandle, UiIntentExecutionAdvanceOutcome,
    UiIntentExecutionDispatchOutcome, UiIntentExecutionTransition,
    UiIntentExecutionTransitionPosture,
};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_runtime::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedFrameExecutionCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::provider::{ConsequenceProvider, ConsequenceProviderControl};
use crate::intent::{
    admission::phase3::world::AdmissionWorld,
    operability::{
        build_consequence_with_provider, build_consequence_with_provider_and_profile,
        consequence_replacement_input, ConsequenceIntent,
    },
};

pub(in crate::intent) struct ConsequenceWorld {
    pub(in crate::intent) interaction: AdmissionWorld,
    query: worth_ui_query_binding::certification::WorthUiOperationLiveTestFixture,
    provider: ConsequenceProviderControl,
    recorder: WorthUiHeadlessRecorder,
    replacement_input: Option<worth_ui_dsl::WorthUiRustAuthoredArtifactInput>,
    next_tick: u64,
}

impl ConsequenceWorld {
    pub(in crate::intent) fn launch(consequences: UiIntentConsequenceContract) -> Self {
        Self::launch_with_capacity(
            consequences,
            UiHeadlessRecorderCapacity::production_default(),
        )
    }

    pub(in crate::intent) fn launch_with_capacity(
        consequences: UiIntentConsequenceContract,
        capacity: UiHeadlessRecorderCapacity,
    ) -> Self {
        Self::launch_with_builder(consequences, capacity, None)
    }

    pub(in crate::intent) fn launch_with_change_profile(
        consequences: UiIntentConsequenceContract,
        profile: worth_ui::facade::rebind::UiChangeProfile,
    ) -> Self {
        Self::launch_with_builder(
            consequences,
            UiHeadlessRecorderCapacity::production_default(),
            Some(profile),
        )
    }

    fn launch_with_builder(
        consequences: UiIntentConsequenceContract,
        capacity: UiHeadlessRecorderCapacity,
        profile: Option<worth_ui::facade::rebind::UiChangeProfile>,
    ) -> Self {
        let mut query = worth_ui_query_binding::certification::WorthUiOperationLiveTestFixture::new(
            "phase4-consequence-world",
        );
        let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
            capacity,
            UiViewportExtentObservation {
                width: 160.0,
                height: 96.0,
            },
        );
        let (provider, control) = ConsequenceProvider::controlled();
        let (application, facts) = match profile {
            Some(profile) => build_consequence_with_provider_and_profile(
                provider,
                query.view(),
                consequences.clone(),
                recorder.clone(),
                profile,
            ),
            None => build_consequence_with_provider(
                provider,
                query.view(),
                consequences.clone(),
                recorder.clone(),
            ),
        };
        let replacement_input = consequence_replacement_input(&facts, consequences);
        let mut interaction =
            AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
        let resource = query.open_resource();
        let mut admission = None;
        let completion = interaction
            .session
            .execute_framework_turn(|turn| {
                turn.query_projection(|owner| {
                    admission = Some(owner.admit_operation_live(resource));
                });
            })
            .expect("initial Query source turn has no mounted presentation conflict");
        drop(completion.into_completion());
        admission
            .expect("Query source owner executed")
            .expect("the exact live resource admits");
        Self {
            interaction,
            query,
            provider: control,
            recorder,
            replacement_input: Some(replacement_input),
            next_tick: 10,
        }
    }

    pub(in crate::intent) fn complete_with_query(&mut self) -> UiIntentConsequenceHandle {
        let consequence = self.mint_query_consequence();
        self.complete_with_consequence(consequence)
    }

    pub(in crate::intent) fn complete_with_consequence(
        &mut self,
        consequence: worth_ui_query_binding::WorthUiCollectionChangeConsequence,
    ) -> UiIntentConsequenceHandle {
        self.provider.supply(consequence);
        self.complete_supplied_outcome()
    }

    pub(in crate::intent) fn complete_without_consequences(&mut self) -> UiIntentConsequenceHandle {
        self.provider.supply_none();
        self.complete_supplied_outcome()
    }

    fn complete_supplied_outcome(&mut self) -> UiIntentConsequenceHandle {
        let admitted = self.interaction.admit_exact_for::<ConsequenceIntent>(0);
        assert!(matches!(
            self.interaction.session.dispatch_admitted_intent(
                admitted,
                super::super::execution_deadline(self.next_tick + 20),
            ),
            UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
        ));
        assert_eq!(
            self.only_transition().posture(),
            UiIntentExecutionTransitionPosture::Started
        );
        self.only_transition()
            .into_consequence()
            .expect("completed product effect returns consequence-only authority")
    }

    fn only_transition(&mut self) -> UiIntentExecutionTransition {
        let tick = self.next_tick;
        self.next_tick += 1;
        let report = match self
            .interaction
            .session
            .advance_intent_executions(super::super::execution_reading(tick))
        {
            UiIntentExecutionAdvanceOutcome::Advanced(report) => report,
            UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
                panic!("monotonic consequence lifecycle advance stopped: {stop:?}")
            }
        };
        let mut transitions = report.into_transitions().into_vec();
        assert_eq!(transitions.len(), 1);
        transitions.pop().unwrap()
    }

    fn mint_query_consequence(
        &mut self,
    ) -> worth_ui_query_binding::WorthUiCollectionChangeConsequence {
        self.query.update_measurement();
        let request = self.query.refresh_request();
        match self
            .interaction
            .session
            .refresh_query_change(request)
            .expect("real Query refresh succeeds")
        {
            worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome::Applied(consequence) => {
                consequence
            }
            worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome::NoSemanticDelivery => {
                panic!("changed Query measurement emits one owner-issued consequence")
            }
        }
    }

    pub(in crate::intent) fn query_change_state(
        &self,
    ) -> worth_ui_query_binding::WorthUiOperationLiveChangeObservation {
        self.interaction
            .session
            .query_change_state(self.query.reference())
            .expect("the exact live resource remains retained")
    }

    pub(in crate::intent) fn provider_calls(&self) -> [usize; 2] {
        self.provider.calls()
    }

    pub(in crate::intent) fn rebind_application(&mut self) {
        let input = self
            .replacement_input
            .take()
            .expect("the consequence world has one authored successor");
        self.interaction.rebind_application_with_input(input);
    }

    pub(in crate::intent) fn publish_compatible_successor_frame(
        &mut self,
    ) -> (
        worth_ui_host_contract::UiMountedFrameIdentity,
        worth_ui_host_contract::UiMountedFrameIdentity,
    ) {
        crate::mounted_geometry_fixture::install_current_occurrence_geometry(
            &mut self.interaction.session,
        );
        let prepared = self
            .interaction
            .session
            .execute_framework_turn(|_| {})
            .expect("no mounted presentation is active")
            .into_execution()
            .unwrap_or_else(|_| panic!("an unchanged ordinary turn remains executable"))
            .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
            .expect("the retained application prepares one compatible successor frame");
        let now_tick = self.next_tick;
        self.next_tick += 1;
        match self.interaction.session.present_prepared_mounted_frame(
            prepared,
            UiPresentationDeadline::at_tick(now_tick + 20),
            now_tick,
        ) {
            UiMountedFrameOutcome::Published(receipt) => {
                let predecessor = receipt
                    .predecessor()
                    .expect("a compatible successor names its published predecessor");
                (predecessor, receipt.frame())
            }
            UiMountedFrameOutcome::Unchanged(_) => {
                panic!("successor setup reused the predecessor instead of publishing a new frame")
            }
            UiMountedFrameOutcome::Reconciled(_) => {
                panic!("successor setup unexpectedly required host reconciliation")
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
                panic!("headless host rejected the compatible successor before effects")
            }
            UiMountedFrameOutcome::InFlight(_) => {
                panic!("headless compatible successor unexpectedly remained in flight")
            }
            UiMountedFrameOutcome::PresentationIndeterminate(_) => {
                panic!("headless compatible successor became indeterminate")
            }
            UiMountedFrameOutcome::RetentionDenied(_) => {
                panic!("mounted retention denied the compatible successor")
            }
            UiMountedFrameOutcome::AdmissionDenied(_) => {
                panic!("mounted presentation admission denied the compatible successor")
            }
            UiMountedFrameOutcome::CompletionDenied(_) => {
                panic!("mounted presentation completion denied the compatible successor")
            }
            UiMountedFrameOutcome::Superseded(_) => {
                panic!("headless compatible successor was unexpectedly superseded")
            }
        }
    }

    pub(in crate::intent) fn transcripts(
        &self,
    ) -> Box<[worth_ui_host_headless::UiHeadlessMountedFrameTranscript]> {
        self.recorder.observed_transcripts()
    }

    pub(in crate::intent) fn drain_transcripts(
        &self,
    ) -> Box<[worth_ui_host_headless::UiHeadlessMountedFrameTranscript]> {
        self.recorder.drain_transcripts()
    }

    pub(in crate::intent) fn shutdown(mut self) {
        let shutdown = self.interaction.session.shutdown();
        assert_eq!(shutdown.intent_execution().active_after(), 0);
        let admission = shutdown.intent_admission();
        assert_eq!(admission.active_after(), 0);
        assert_eq!(admission.retained_candidates_after(), 0);
        assert_eq!(admission.retained_payloads_after(), 0);
        assert!(shutdown.rebind().is_empty());
        assert!(shutdown.mounted_presentation().is_empty());
        assert!(matches!(
            self.query
                .close_retirement(shutdown.into_operation_live_retirement()),
            worth_ui_query_binding::WorthUiOperationLiveRetirementCloseOutcome::Closed(_)
        ));
    }
}
