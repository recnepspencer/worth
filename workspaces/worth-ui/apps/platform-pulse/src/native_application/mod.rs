use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui::facade::source::WorthUiSourcePackageRevision;

use crate::application::PlatformPulsePreparationDenial;
use crate::lifecycle_observation_publication::{
    PlatformPulseObservationPublicationDenial, PlatformPulseObservationPublisher,
};
use crate::source_watch::PlatformPulseSourceWatch;
use crate::visual_identity_execution::{
    PlatformPulseVisualExecutionDenial, PlatformPulseVisualIdentityExecution,
};

mod composition;
mod first_frame;
mod frame_execution_diagnostic;
mod frame_presentation;
mod input;
mod intent;
mod layout;
mod lifecycle;
mod product_copy;
mod product_story;
mod projection;
mod query;
mod readiness;
mod rebind;
mod source_rebind;
mod terminal_error;

pub(crate) use composition::PlatformPulseApplication;

use frame_presentation::PlatformPulsePendingFramePresentation;
use projection::PlatformPulseProjectionRebindDenial;
use terminal_error::PlatformPulseTerminalError;

enum PlatformPulsePendingManagedRebind {
    Projection(query::PlatformPulsePendingProjection),
    Source(WorthUiSourcePackageRevision),
    IntentPosture(intent::PlatformPulsePendingIntentPosture),
    IntentConsequence(intent::PlatformPulsePendingIntentConsequence),
    PortalDismissal,
}

pub(crate) struct PlatformPulseApplicationRuntime {
    initial_source: Option<WorthUiSourcePackageRevision>,
    shell: Option<WorthUiNativeApplicationShell>,
    source_watch: Option<PlatformPulseSourceWatch>,
    query_watch: Option<crate::query_source::PlatformPulseExternalValueWatch>,
    query_lifecycle: Option<crate::query_source::PlatformPulseQueryLifecycle>,
    intent_watch: Option<worth_ui_platform_pulse::intent::PlatformPulseIntentInputWatch>,
    intent_gate: Option<worth_ui_platform_pulse::intent::PlatformPulseExecutorGate>,
    intent_action_owner: Option<worth_ui_platform_pulse::intent::PlatformPulseActionPortOwner>,
    pending_query_actions: Vec<PlatformPulsePendingQueryAction>,
    pending_query_denial_story: Option<
        worth_ui_platform_pulse::observation_contract::PlatformPulseQueryActionPreconditionDenial,
    >,
    pending_frame_presentation: Option<PlatformPulsePendingFramePresentation>,
    pending_managed_rebind: Option<PlatformPulsePendingManagedRebind>,
    pending_intent_postures: std::collections::VecDeque<intent::PlatformPulsePreparedIntentPosture>,
    pending_intent_execution_transitions:
        std::collections::VecDeque<worth_ui::facade::intent::UiIntentExecutionTransition>,
    intent_evidence_index: intent::PlatformPulseIntentEvidenceIndex,
    native_input: input::PlatformPulseNativeInputIngress,
    publisher: PlatformPulseObservationPublisher,
    terminal_error: Option<PlatformPulseTerminalError>,
    observation_error: Option<PlatformPulseObservationPublicationDenial>,
    terminal_reported: bool,
    visual_identity: PlatformPulseVisualIdentityExecution,
    intent_clock: intent::PlatformPulseIntentClock,
    presentation_tick: u64,
    product_story: product_story::PlatformPulseProductStory,
}

struct PlatformPulsePendingQueryAction {
    reference: worth_ui_platform_pulse::intent::PlatformPulseActionAttemptReference,
    evidence_reference: worth_ui::facade::inspection::UiIntentEvidenceReference,
    projection: worth_ui_platform_pulse::observation_contract::PlatformPulseQueryProjectionEvidence,
}

impl PlatformPulseApplicationRuntime {
    fn refresh_product_story(&mut self, shell: &mut WorthUiNativeApplicationShell) -> bool {
        match self.product_story.refresh_runtime(shell) {
            Ok(()) => true,
            Err(denial) => {
                self.fail(PlatformPulseTerminalError::ProductCopy(denial), Ok(()));
                false
            }
        }
    }

    fn publish_source_story(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        sequence: u64,
    ) -> bool {
        match self.product_story.publish_source(shell, sequence) {
            Ok(()) => true,
            Err(denial) => {
                self.fail(PlatformPulseTerminalError::ProductCopy(denial), Ok(()));
                false
            }
        }
    }

    fn publish_query_denial_story(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        denial: worth_ui_platform_pulse::observation_contract::PlatformPulseQueryActionPreconditionDenial,
    ) -> bool {
        match self.product_story.publish_query_denial(shell, denial) {
            Ok(()) => true,
            Err(error) => {
                self.fail(PlatformPulseTerminalError::ProductCopy(error), Ok(()));
                false
            }
        }
    }

    fn fail(
        &mut self,
        error: PlatformPulseTerminalError,
        observation: Result<(), PlatformPulseObservationPublicationDenial>,
    ) {
        self.terminal_error = Some(error);
        self.observation_error = observation.err();
    }

    fn advance_visual_identity(&mut self) {
        let Some(shell) = self.shell.as_mut() else {
            return;
        };
        let result = self.visual_identity.advance(
            shell,
            &self.publisher,
            &mut self.presentation_tick,
            std::time::Instant::now(),
        );
        if let Err(denial) = result {
            self.fail_visual_identity(denial);
        }
    }

    fn fail_visual_identity(&mut self, denial: PlatformPulseVisualExecutionDenial) {
        let observation = self.publisher.visual_identity_failure();
        self.fail(
            PlatformPulseTerminalError::VisualIdentity(denial),
            observation,
        );
    }

    pub(crate) fn report_terminal_error(&mut self) {
        if self.terminal_reported {
            return;
        }
        if let Some(error) = &self.terminal_error {
            eprintln!("WORTH UI platform pulse stopped: {error}");
        }
        if let Some(error) = self.observation_error {
            eprintln!("WORTH UI platform pulse could not publish terminal evidence: {error:?}");
        }
        self.terminal_reported = true;
    }
}

impl Drop for PlatformPulseApplicationRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown_product();
    }
}

pub(crate) fn publish_preparation_failure(
    publisher: &PlatformPulseObservationPublisher,
    denial: &PlatformPulsePreparationDenial,
) -> Result<(), PlatformPulseObservationPublicationDenial> {
    match denial {
        PlatformPulsePreparationDenial::WatcherStart(denial)
        | PlatformPulsePreparationDenial::InitialSourceSettlement(denial) => {
            publisher.filesystem_watcher_failure(denial)
        }
        PlatformPulsePreparationDenial::CapabilityApplication(denial) => {
            publisher.application_preparation_failure(denial)
        }
        PlatformPulsePreparationDenial::InitialSourceLowering(denial) => {
            publisher.candidate_submission_failure(denial)
        }
        PlatformPulsePreparationDenial::QueryInstallation(_)
        | PlatformPulsePreparationDenial::QueryRegistration(_)
        | PlatformPulsePreparationDenial::QueryViewRegistration(_) => {
            publisher.query_preparation_failure()
        }
        PlatformPulsePreparationDenial::IntentInput(_)
        | PlatformPulsePreparationDenial::IntentFact(_)
        | PlatformPulsePreparationDenial::IntentDefinition(_)
        | PlatformPulsePreparationDenial::IntentProvider(_) => {
            publisher.intent_preparation_failure()
        }
    }
}
