use worth_ui::facade::app::WorthUiApplicationPreparationDenial;
use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUi, WorthUiApplicationBuilder,
};
use worth_ui::facade::intent::{
    UiIntentApplicationFactRegistrationError, UiIntentDefinitionRegistrationError,
    UiIntentExecutionBindingPreparationDenial,
};
use worth_ui::facade::query_binding::WorthUiProjectionRegistrationError;
use worth_ui::facade::source::{
    UiSourceRebindAttemptFailure, WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher,
    WorthUiFilesystemWatcherDenial, WorthUiSourcePackageRevision,
};

use crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration;
use crate::query_source::{
    InstalledPlatformPulseQuery, PlatformPulseExternalValueWatch,
    PlatformPulseQueryInstallationDenial, PlatformPulseQueryLifecycle,
};
use worth_ui_platform_pulse::intent::{
    platform_pulse_action_confirmation_fact, platform_pulse_action_definition,
    platform_pulse_action_mutability_fact, platform_pulse_action_policy_fact,
    platform_pulse_action_readiness_fact, platform_pulse_action_revision_fact,
    platform_pulse_close_portal_confirmation_fact, platform_pulse_close_portal_definition,
    platform_pulse_close_portal_mutability_fact, platform_pulse_close_portal_policy_fact,
    platform_pulse_close_portal_readiness_fact, platform_pulse_open_portal_definition,
    platform_pulse_query_denial_fact, PlatformPulseActionPortOwner, PlatformPulseActionProvider,
    PlatformPulseExecutorGate, PlatformPulseIntentInputInstallation,
    PlatformPulseIntentInputRecord, PlatformPulseIntentInputWatch,
    PlatformPulseIntentInputWatchDenial,
};
mod command_story;
mod mosaic;
mod presentation;

use mosaic::register_mosaic;
use presentation::{register_appearance, register_structure, visual_inspection_policy};

pub(crate) struct PreparedPlatformPulseComposition {
    pub(crate) theme_watcher: crate::theme_preference::PlatformPulseThemePreferenceWatch,
    pub(crate) builder:
        WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    pub(crate) watcher: WorthUiFilesystemSourceWatcher,
    pub(crate) initial_source: WorthUiSourcePackageRevision,
    pub(crate) query_lifecycle: PlatformPulseQueryLifecycle,
    pub(crate) query_watcher: PlatformPulseExternalValueWatch,
    pub(crate) intent_watcher: PlatformPulseIntentInputWatch,
    pub(crate) intent_gate: PlatformPulseExecutorGate,
    pub(crate) intent_action_owner: PlatformPulseActionPortOwner,
}

#[derive(Debug)]
pub(crate) enum PlatformPulsePreparationDenial {
    ThemePreference(crate::theme_preference::PlatformPulseThemePreferenceDenial),
    WatcherStart(WorthUiFilesystemWatcherDenial),
    InitialSourceSettlement(WorthUiFilesystemWatcherDenial),
    CapabilityApplication(Box<WorthUiApplicationPreparationDenial>),
    InitialSourceLowering(UiSourceRebindAttemptFailure),
    QueryInstallation(Box<PlatformPulseQueryInstallationDenial>),
    QueryRegistration(WorthUiProjectionRegistrationError),
    IntentInput(PlatformPulseIntentInputWatchDenial),
    IntentFact(UiIntentApplicationFactRegistrationError),
    IntentDefinition(UiIntentDefinitionRegistrationError),
    IntentProvider(UiIntentExecutionBindingPreparationDenial),
    Appearance(presentation::PlatformPulseAppearanceRegistrationDenial),
}

pub(crate) fn prepare_composition(
    launch: &AdmittedPlatformPulseLaunchConfiguration,
) -> Result<PreparedPlatformPulseComposition, PlatformPulsePreparationDenial> {
    let theme_watcher = crate::theme_preference::PlatformPulseThemePreferenceWatch::open(
        launch.intent_source_root(),
    )
    .map_err(PlatformPulsePreparationDenial::ThemePreference)?;
    let query = crate::query_source::install(launch.query_source_root())
        .map_err(|denial| PlatformPulsePreparationDenial::QueryInstallation(Box::new(denial)))?;
    let intent = match PlatformPulseIntentInputInstallation::open(launch.intent_source_root()) {
        Ok(intent) => intent,
        Err(denial) => {
            query.shutdown();
            return Err(PlatformPulsePreparationDenial::IntentInput(denial));
        }
    };
    let (intent_initial, intent_watcher) = intent.into_parts();
    let intent_gate =
        PlatformPulseExecutorGate::at(intent_initial.revision(), intent_initial.executor_held());
    let (intent_port, intent_action_owner) =
        worth_ui_platform_pulse::intent::PlatformPulseActionPort::bounded();
    let intent_provider = PlatformPulseActionProvider::new(intent_port, intent_gate.clone());
    let provider = WorthUiFilesystemSourceProvider::new(launch.source_root());
    let mut watcher = match WorthUiFilesystemSourceWatcher::start(provider) {
        Ok(watcher) => watcher,
        Err(denial) => {
            query.shutdown();
            let _ = intent_watcher.shutdown();
            return Err(PlatformPulsePreparationDenial::WatcherStart(denial));
        }
    };
    let InstalledPlatformPulseQuery {
        registration,
        lifecycle: query_lifecycle,
        watcher: query_watcher,
    } = query;
    let result = (|| {
        let snapshot = watcher
            .take_initial_snapshot()
            .map_err(PlatformPulsePreparationDenial::InitialSourceSettlement)?;
        let initial_source = snapshot.source_revision().clone();
        let fonts = presentation::PulseFonts::admit();
        let capability_builder = builder(
            registration.clone(),
            &intent_initial,
            intent_provider.clone(),
            &fonts,
        )?;
        let capability_app = capability_builder.freeze().map_err(|denial| {
            PlatformPulsePreparationDenial::CapabilityApplication(Box::new(denial))
        })?;
        let submission = snapshot
            .attempt_source_rebind(capability_app.capabilities())
            .into_candidate_submission()
            .map_err(PlatformPulsePreparationDenial::InitialSourceLowering)?;
        drop(capability_app);
        builder(registration, &intent_initial, intent_provider, &fonts).map(|builder| {
            (
                builder.with_candidate_submission(submission),
                initial_source,
            )
        })
    })();
    match result {
        Ok((builder, initial_source)) => Ok(PreparedPlatformPulseComposition {
            theme_watcher,
            builder,
            watcher,
            initial_source,
            query_lifecycle,
            query_watcher,
            intent_watcher,
            intent_gate,
            intent_action_owner,
        }),
        Err(denial) => {
            let _ = watcher.shutdown();
            let _ = query_watcher.shutdown();
            let _ = query_lifecycle.close();
            let _ = intent_watcher.shutdown();
            Err(denial)
        }
    }
}

impl std::fmt::Display for PlatformPulsePreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThemePreference(denial) => write!(formatter, "theme preference: {denial:?}"),
            Self::WatcherStart(denial) => write!(formatter, "watcher start: {denial:?}"),
            Self::InitialSourceSettlement(denial) => {
                write!(formatter, "initial source settlement: {denial:?}")
            }
            Self::CapabilityApplication(denial) => {
                write!(formatter, "capability application: {denial:?}")
            }
            Self::InitialSourceLowering(denial) => {
                write!(formatter, "initial source lowering: {denial:?}")
            }
            Self::QueryInstallation(denial) => {
                write!(formatter, "Query installation: {denial}")
            }
            Self::QueryRegistration(denial) => {
                write!(formatter, "Query registration: {denial:?}")
            }
            Self::IntentInput(denial) => write!(formatter, "intent input: {denial}"),
            Self::IntentFact(denial) => write!(formatter, "intent fact: {denial:?}"),
            Self::IntentDefinition(denial) => write!(formatter, "intent definition: {denial:?}"),
            Self::IntentProvider(denial) => write!(formatter, "intent provider: {denial:?}"),
            Self::Appearance(denial) => write!(formatter, "appearance: {denial}"),
        }
    }
}

fn builder(
    registration: worth_ui::facade::query_binding::UiApplicationScalarProjectionRegistration,
    intent: &PlatformPulseIntentInputRecord,
    provider: PlatformPulseActionProvider,
    fonts: &presentation::PulseFonts,
) -> Result<
    WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    PlatformPulsePreparationDenial,
> {
    let builder = register_structure(
        register_mosaic(
            WorthUi::app()
                .with_font_collection(std::sync::Arc::clone(&fonts.collection))
                .with_change_profile(dashboard_change_profile()),
        ),
        fonts,
    );
    let builder = register_appearance(builder)
        .map_err(PlatformPulsePreparationDenial::Appearance)?
        .register_intent_boolean_fact(platform_pulse_close_portal_mutability_fact(), true)
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(platform_pulse_close_portal_readiness_fact(), true)
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(platform_pulse_close_portal_policy_fact(), true)
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(platform_pulse_close_portal_confirmation_fact(), false)
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(platform_pulse_action_mutability_fact(), intent.mutable())
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(platform_pulse_action_readiness_fact(), intent.ready())
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(platform_pulse_action_policy_fact(), intent.policy_allowed())
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(
            platform_pulse_action_confirmation_fact(),
            intent.confirmation_required(),
        )
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_unsigned64_fact(platform_pulse_action_revision_fact(), intent.revision())
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_boolean_fact(
            platform_pulse_query_denial_fact(),
            intent.query_denial_requested(),
        )
        .map_err(PlatformPulsePreparationDenial::IntentFact)?
        .register_intent_definition(platform_pulse_action_definition())
        .map_err(PlatformPulsePreparationDenial::IntentDefinition)?
        .register_intent_provider(provider)
        .map_err(PlatformPulsePreparationDenial::IntentProvider)?
        .register_runtime_service_intent_definition(platform_pulse_open_portal_definition())
        .map_err(PlatformPulsePreparationDenial::IntentDefinition)?
        .register_runtime_service_intent_definition(platform_pulse_close_portal_definition())
        .map_err(PlatformPulsePreparationDenial::IntentDefinition)?;
    command_story::register(builder)?
        .register_application_scalar_projection(registration)
        .map(|builder| builder.with_visual_inspection_policy(visual_inspection_policy()))
        .map_err(PlatformPulsePreparationDenial::QueryRegistration)
}

fn dashboard_change_profile() -> worth_ui::facade::rebind::UiChangeProfile {
    use worth_ui::facade::observation::{UiObservationProfile, UiObservationProfileInput};
    use worth_ui::facade::rebind::{UiChangeProfile, UiRebindProfile};
    // The authored dashboard is 77 KiB across eleven modules. Source ingress
    // retains the package for a candidate, including unchanged modules.
    let observation = UiObservationProfile::bounded(UiObservationProfileInput {
        admitted_per_turn: 8,
        retained_bytes_per_turn: 128 * 1024,
        queued_during_effecting_rebind: 16,
    })
    .expect("dashboard source admission has a nonzero bounded budget");
    let baseline = UiRebindProfile::platform_pulse();
    let mut budget = baseline.budget();
    // Comparison visits the full declaration set, not only the edited role.
    // The dashboard has roughly 375 declarations plus its eleven modules.
    budget.comparison_structural_entries = 512;
    UiChangeProfile::new(
        observation,
        UiRebindProfile::bounded(budget, baseline.concurrency())
            .expect("dashboard comparison retains bounded admission"),
    )
}
