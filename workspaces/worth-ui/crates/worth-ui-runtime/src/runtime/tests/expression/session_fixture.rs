use crate::facade::entry::{WorthUiApplicationBuilder, WorthUiCertificationApplicationTransition};
use crate::facade::expression::UiExpressionWorkCounters;
use crate::facade::intent::UiIntentApplicationFact;
use crate::facade::lifecycle::WorthUiApplicationPreparationDenial;
use crate::facade::{WorthUiActiveApplicationSession, WorthUiApp};
use crate::runtime::tests::active_application_session_test_support::component_builder;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};

pub(crate) const READY: &str = "app.ready";
pub(crate) const COUNT: &str = "app.count";
pub(crate) const LABEL: &str = "app.label";

const COMPONENT: &str =
    "component workspace.component.active_session_current { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }";

/// The registered application facts every expression fixture reads.
pub(crate) fn builder() -> WorthUiApplicationBuilder {
    component_builder()
        .register_intent_boolean_fact(
            UiIntentApplicationFact::boolean(READY).expect("valid fact identity"),
            false,
        )
        .expect("ready fact registers")
        .register_intent_unsigned64_fact(
            UiIntentApplicationFact::unsigned64(COUNT).expect("valid fact identity"),
            3,
        )
        .expect("count fact registers")
        .register_intent_text_fact(
            UiIntentApplicationFact::text(LABEL, 64).expect("valid fact identity"),
            "idle",
        )
        .expect("label fact registers")
}

/// The full authored source: the shared component plus `declarations`.
pub(crate) fn source(declarations: &str) -> String {
    format!("{COMPONENT}\n{declarations}")
}

/// Freezes an application whose authored source carries `declarations`.
pub(crate) fn prepare(
    declarations: &str,
) -> Result<WorthUiApp, WorthUiApplicationPreparationDenial> {
    prepare_with(builder, declarations)
}

/// Freezes the application `configure` builds, with `declarations` authored.
pub(crate) fn prepare_with(
    configure: impl Fn() -> WorthUiApplicationBuilder,
    declarations: &str,
) -> Result<WorthUiApp, WorthUiApplicationPreparationDenial> {
    let snapshot = configure()
        .freeze()
        .map(WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("fixture capabilities prepare");
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("expression-fixture")
            .with_file("app/main.wui", source(declarations)),
        [WorthUiWatcherEvent::provider_revision("expression-fixture")],
        snapshot.capabilities(),
    );
    configure()
        .with_candidate_submission(submission)
        .freeze()
        .map(WorthUiCertificationApplicationTransition::activate_builder_host)
}

/// Launches a session whose authored source carries `declarations`.
pub(crate) fn launch(declarations: &str) -> WorthUiActiveApplicationSession {
    prepare(declarations)
        .expect("expression fixture prepares")
        .launch()
        .expect("expression fixture launches")
}

/// The work the expression owner of `session` has done since `before`.
pub(crate) fn since(
    session: &WorthUiActiveApplicationSession,
    before: UiExpressionWorkCounters,
) -> UiExpressionWorkCounters {
    let now = session.expression_work_counters();
    UiExpressionWorkCounters {
        evaluations: now.evaluations - before.evaluations,
        settled_without_evaluation: now.settled_without_evaluation
            - before.settled_without_evaluation,
        operand_probes: now.operand_probes - before.operand_probes,
        index_hits: now.index_hits - before.index_hits,
        published_changes: now.published_changes - before.published_changes,
        suppressed_unchanged: now.suppressed_unchanged - before.suppressed_unchanged,
        stale_completions: now.stale_completions - before.stale_completions,
    }
}
