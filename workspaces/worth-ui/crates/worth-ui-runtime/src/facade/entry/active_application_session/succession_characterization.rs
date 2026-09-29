//! What a generation succession leaves behind, read the same way for every
//! writer: the work it spent, and whether the pointer snapshot and standing
//! facts it committed are what a fresh observation of the committed state
//! decides. The characterization tests of each writer pin these exactly, so
//! a later refactor of succession proves it changes none of them.

use crate::facade::WorthUiActiveApplicationSession;

/// The work counters a succession moves, each read from its own owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) struct SuccessionWork {
    pub(in crate::facade::entry) reobservations: u64,
    pub(in crate::facade::entry) operand_probes: u64,
    pub(in crate::facade::entry) index_hits: u64,
    pub(in crate::facade::entry) evaluations: u64,
    pub(in crate::facade::entry) appearance_batches: u64,
}

impl SuccessionWork {
    /// The counters of `session` as they stand.
    pub(in crate::facade::entry) fn read(session: &WorthUiActiveApplicationSession) -> Self {
        let expressions = session.expression_work_counters();
        Self {
            reobservations: session
                .intent_admission_metrics()
                .operability_reobservations(),
            operand_probes: expressions.operand_probes,
            index_hits: expressions.index_hits,
            evaluations: expressions.evaluations,
            appearance_batches: session
                .presentation
                .queued_appearance_invalidation_batches(),
        }
    }

    /// The work `session` has done since these counters were read.
    pub(in crate::facade::entry) fn since(self, session: &WorthUiActiveApplicationSession) -> Self {
        let now = Self::read(session);
        Self {
            reobservations: now.reobservations - self.reobservations,
            operand_probes: now.operand_probes - self.operand_probes,
            index_hits: now.index_hits - self.index_hits,
            evaluations: now.evaluations - self.evaluations,
            appearance_batches: now.appearance_batches - self.appearance_batches,
        }
    }
}

/// The part of one pointer row's operability a fresh observation must
/// reproduce: its generation, node, route, whether it is operable, and the
/// product decision, or why no observation was available.
type Decided = Result<
    (
        crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        crate::graph::UiGraphNodeIdentity,
        String,
        bool,
        Option<crate::runtime::intent::UiIntentOperabilityDecision>,
    ),
    String,
>;

fn decided(
    observation: Result<
        &crate::runtime::intent::UiIntentStandingOperabilityObservation,
        &crate::runtime::intent::UiIntentStandingOperabilityUnavailable,
    >,
) -> Decided {
    observation
        .map(|observation| {
            (
                observation.generation().clone(),
                observation.graph_node(),
                observation.route().to_owned(),
                observation.is_operable(),
                observation.product_decision().cloned(),
            )
        })
        .map_err(|unavailable| format!("{unavailable:?}"))
}

/// Each hovered row of the committed pointer snapshot beside what a fresh
/// observation of the committed state decides for the same target, in row
/// order. Empty when no snapshot is committed.
pub(in crate::facade::entry) fn pointer_rows(
    session: &WorthUiActiveApplicationSession,
) -> Vec<(Decided, Decided)> {
    let Some(snapshot) = session.pointer_affordance_snapshot.as_ref() else {
        return Vec::new();
    };
    let prepared = session.application.prepared_authority();
    let active = session.active_generation_identity();
    let host_time = session.observation_clock.as_ref().map(|clock| {
        worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(
            clock.sample_millis(),
        )
    });
    snapshot
        .active_projections()
        .filter_map(|row| Some((row.presented_target()?, row.operability()?)))
        .map(|(target, committed)| {
            let fresh = crate::runtime::intent::observe_activation_operability(
                target,
                session.intent_read_owners(prepared, &active),
                &session.intent_confirmation,
                host_time,
            );
            (decided(committed), decided(fresh.as_ref()))
        })
        .collect()
}

/// Asserts a pointer snapshot is committed exactly when `present`, that it
/// names the active generation, and that each hovered row decides what a
/// fresh observation of the committed state decides.
pub(in crate::facade::entry) fn assert_pointer_is_fresh(
    session: &WorthUiActiveApplicationSession,
    present: bool,
) {
    let snapshot = session.pointer_affordance_snapshot.as_ref();
    assert_eq!(snapshot.is_some(), present, "pointer snapshot presence");
    if let Some(snapshot) = snapshot {
        assert_eq!(snapshot.generation(), &session.active_generation_identity());
    }
    for (committed, fresh) in pointer_rows(session) {
        assert_eq!(
            committed, fresh,
            "the committed pointer row decides what a fresh observation decides"
        );
    }
}

/// Each standing fact's recorded decision beside what re-observing it over
/// the committed state decides.
pub(in crate::facade::entry) fn standing_rows(
    session: &WorthUiActiveApplicationSession,
) -> Vec<(
    Option<crate::runtime::intent::UiIntentOperabilityDecision>,
    Option<crate::runtime::intent::UiIntentOperabilityDecision>,
)> {
    let prepared = session.application.prepared_authority();
    let active = session.active_generation_identity();
    session
        .intent_admission
        .operability_standing_snapshot()
        .map(|snapshot| snapshot.facts())
        .unwrap_or_default()
        .iter()
        .map(|fact| {
            (
                Some(fact.decision().clone()),
                crate::runtime::intent::reobserve_standing_fact(
                    fact,
                    session.intent_read_owners(prepared, &active),
                ),
            )
        })
        .collect()
}

/// Asserts `facts` standing facts are committed and each records what a
/// fresh re-observation of the committed state decides.
pub(in crate::facade::entry) fn assert_standing_is_fresh(
    session: &WorthUiActiveApplicationSession,
    facts: usize,
) {
    let rows = standing_rows(session);
    assert_eq!(rows.len(), facts, "standing fact count");
    for (committed, fresh) in rows {
        assert_eq!(
            committed, fresh,
            "the standing fact records what a fresh re-observation decides"
        );
    }
}

/// Asserts the retained appearance owner snapshot is committed exactly when
/// `present` and then names the active generation, and that the overlay
/// bindings follow the active generation.
pub(in crate::facade::entry) fn assert_owners_follow(
    session: &WorthUiActiveApplicationSession,
    present: bool,
) {
    let active = session.active_generation_identity();
    let owners = session.appearance_owner_snapshot.as_ref();
    assert_eq!(owners.is_some(), present, "appearance owner snapshot presence");
    if let Some(owners) = owners {
        assert_eq!(owners.generation(), &active);
    }
    // Exporting requires the lifecycle's own generation, so it answers
    // whether the overlay bindings follow the active one.
    assert!(
        session
            .authored_overlay_bindings
            .exports(&active, None)
            .is_ok(),
        "the overlay bindings follow the active generation"
    );
}
