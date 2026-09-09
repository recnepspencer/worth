use crate::application::{
    prepare_composition, PlatformPulsePreparationDenial, PreparedPlatformPulseComposition,
};
use crate::source_watch::PlatformPulseSourceWatch;
use crate::visual_identity_execution::PlatformPulseVisualIdentityExecution;
use worth_ui_platform_pulse::product_world::PlatformPulseMosaicSurface;

pub(crate) struct PlatformPulseApplication {
    launch: crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration,
    publisher: crate::lifecycle_observation_publication::PlatformPulseObservationPublisher,
}

impl PlatformPulseApplication {
    pub(crate) fn new(
        launch: crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration,
        publisher: crate::lifecycle_observation_publication::PlatformPulseObservationPublisher,
    ) -> Self {
        Self { launch, publisher }
    }
}

impl worth_ui_native_platform::UiNativeApplicationDefinition for PlatformPulseApplication {
    fn prepare(
        self,
        mut preparation: worth_ui_native_platform::UiNativeApplicationPreparation,
    ) -> worth_ui_native_platform::UiNativeApplicationPreparationOutcome {
        let composition = match prepare_composition(&self.launch) {
            Ok(composition) => composition,
            Err(denial) => {
                publish_preparation_failure(&self.publisher, &denial);
                return preparation.deny(
                    worth_ui_native_platform::UiNativeApplicationPreparationDenialCause::ApplicationRejected,
                );
            }
        };
        let (builder, runtime) =
            super::PlatformPulseApplicationRuntime::from_composition(composition, self.publisher);
        let Some(presentation_async) = crate::query_source::install_native_presentation_async()
        else {
            let _ = runtime.publisher.query_preparation_failure();
            drop(runtime);
            return preparation.deny(
                worth_ui_native_platform::UiNativeApplicationPreparationDenialCause::ApplicationRejected,
            );
        };
        if let Err(cause) = preparation.install_presentation_async(presentation_async) {
            return preparation.deny(cause);
        }
        if let Err(cause) = preparation.install_application_composition(builder) {
            return preparation.deny(cause);
        }
        if let Err(cause) =
            preparation.install_native_surface_declaration(PlatformPulseMosaicSurface::Main.id())
        {
            return preparation.deny(cause);
        }
        if let Err(cause) = preparation.install_application_runtime(runtime) {
            return preparation.deny(cause);
        }
        preparation.complete()
    }
}

impl super::PlatformPulseApplicationRuntime {
    pub(crate) fn from_composition(
        composition: PreparedPlatformPulseComposition,
        publisher: crate::lifecycle_observation_publication::PlatformPulseObservationPublisher,
    ) -> (
        worth_ui::facade::app::WorthUiApplicationBuilder<
            worth_ui::facade::app::UiChangeProfileInstalled,
            worth_ui::facade::app::UiIntentWiringSatisfied,
        >,
        Self,
    ) {
        let PreparedPlatformPulseComposition {
            builder,
            watcher,
            initial_source,
            query_lifecycle,
            query_watcher,
            intent_watcher,
            intent_gate,
            intent_action_owner,
        } = composition;
        let runtime = Self {
            initial_source: Some(initial_source),
            shell: None,
            source_watch: Some(PlatformPulseSourceWatch::spawn(watcher)),
            query_watch: Some(query_watcher),
            query_lifecycle: Some(query_lifecycle),
            intent_watch: Some(intent_watcher),
            intent_gate: Some(intent_gate),
            intent_action_owner: Some(intent_action_owner),
            pending_query_actions: Vec::new(),
            pending_query_denial_story: None,
            pending_frame_presentation: None,
            pending_managed_rebind: None,
            pending_intent_postures: std::collections::VecDeque::new(),
            pending_intent_execution_transitions: std::collections::VecDeque::new(),
            intent_evidence_index: super::intent::PlatformPulseIntentEvidenceIndex::new(),
            native_input: super::input::PlatformPulseNativeInputIngress::default(),
            publisher,
            terminal_error: None,
            observation_error: None,
            terminal_reported: false,
            visual_identity: PlatformPulseVisualIdentityExecution::new(),
            intent_clock: super::intent::PlatformPulseIntentClock::new(),
            presentation_tick: 0,
            product_story: super::product_story::PlatformPulseProductStory::default(),
        };
        (builder, runtime)
    }
}

fn publish_preparation_failure(
    publisher: &crate::lifecycle_observation_publication::PlatformPulseObservationPublisher,
    denial: &PlatformPulsePreparationDenial,
) {
    if let Err(publication) =
        crate::native_application::publish_preparation_failure(publisher, denial)
    {
        eprintln!(
            "WORTH UI native Pulse preparation denial could not be observed: {publication:?}"
        );
    }
}
