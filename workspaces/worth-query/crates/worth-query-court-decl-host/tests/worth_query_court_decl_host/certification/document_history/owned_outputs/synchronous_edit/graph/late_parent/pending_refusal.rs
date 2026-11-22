//! Pending domain refusals (PR C C4, §1.7; the #229 addendum): a route
//! derivation that refuses in its domain while a later authored stage remains
//! waits on its proofs (what its decision read). A later stage that changes
//! one retries it; one still pending where it is needed reports its original
//! refusal, the last one if a retry refused again.
use super::*;
use family::c3b::guarded_input;
use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

/// Every edit stage roots `guarded-input(a)`: a + b, refused while b is odd.
pub struct Guarded;
impl ApplicationConnectionIdentity for Guarded {
    const IDENTITY: &'static str = "owned-outputs.late-parent.guarded-start";
}
type Entry = super::super::super::super::program::Entry;
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, guarded_input::Feature> for Guarded {}
type GuardedConnection = ApplicationConnectionRef<
    Schema,
    Entry,
    super::super::super::super::program::EntryOutput,
    guarded_input::Feature,
    guarded_input::InputPort,
    Guarded,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Guarded {
    type Source = edit::Binding;
    type Demand = guarded_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(guarded_input::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Guarded> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<guarded_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Guarded as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub type Tree = ApplicationOutputGraph<GuardedConnection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.guarded-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.guarded-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// The outcome of `stages` (one envelope when `envelope`, else one commit
/// each) on a world with a = 2, b = 3, and `guarded:a` after it.
fn run(stages: &[(&str, u64)], envelope: bool) -> (Vec<Outcome>, Option<u64>) {
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(3),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let edits: Vec<&[(&str, u64)]> = if envelope {
        vec![stages]
    } else {
        stages.chunks(1).collect()
    };
    let mut outcomes = Vec::new();
    for (ordinal, stages) in edits.into_iter().enumerate() {
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("pending-refusal:{envelope}:{ordinal}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        for (key, value) in stages {
            edit.push_stage::<Tree, _>(edit::Intent {
                key: (*key).into(),
                value: *value,
            })
            .unwrap();
        }
        outcomes.push(
            edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap(),
        );
    }
    let guarded = request
        .query(query::first_output::Read("guarded:a".into()))
        .execute()
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value));
    (outcomes, guarded)
}

/// The domain reason an outcome refused with.
fn refused(outcome: &Outcome) -> Option<String> {
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        return None;
    };
    cause
        .coupled_producer_denial()
        .and_then(|denial| denial.domain_reason().map(str::to_owned))
}

/// (e) Base-order equality. The addendum's positive shape: stage 0 (a = 4) roots guarded-input(a),
/// which over b = 3 (odd) refuses, and stage 1 writes b = 8. One commit per
/// stage refuses at stage 0. In the envelope no barrier needs the route, so
/// base order derives it after the last stage, over b = 8: 4 + 8 = 12, the
/// world its final derivations build (the addendum's oracle), built here
/// first (b = 8 committed, then a = 4). The route never runs on the
/// intermediate state, so no refusal is pending here.
#[test]
fn a_pending_refusal_a_later_stage_fixes_commits_the_final_derivation() {
    let (sequential, _) = run(&[("a", 4), ("b", 8)], false);
    assert_eq!(
        refused(&sequential[0]).as_deref(),
        Some("DerivedValueOutOfRange"),
        "one commit per stage refuses at stage 0"
    );
    let (_, oracle) = run(&[("b", 8), ("a", 4)], false);
    let (outcome, guarded) = run(&[("a", 4), ("b", 8)], true);
    assert!(matches!(outcome[0], Outcome::Committed(_)), "{:?}", outcome[0]);
    assert_eq!(oracle, Some(12));
    assert_eq!(guarded, oracle);
}

/// Stage 1 writes b = 5, still odd: guarded-input(a) refuses the same way
/// over the final state, and that refusal is reported unchanged. Identical
/// with deferral disabled: base order runs the route after the last stage,
/// so it is never pending here.
#[test]
fn a_pending_refusal_still_refused_at_the_end_reports_its_refusal() {
    let (outcome, guarded) = run(&[("a", 4), ("b", 5)], true);
    assert_eq!(refused(&outcome[0]).as_deref(), Some("DerivedValueOutOfRange"));
    assert_eq!(guarded, None);
}

/// Stage 1 writes b = 101: guarded-input(a) refuses differently over the final
/// state, and that last refusal is the one reported. Identical with deferral
/// disabled, for the same reason (the within-drive retry is
/// `pending_retry.rs`).
#[test]
fn a_retry_that_refuses_differently_reports_the_last_refusal() {
    let (outcome, guarded) = run(&[("a", 4), ("b", 101)], true);
    assert_eq!(refused(&outcome[0]).as_deref(), Some("PreserveMissingOutput"));
    assert_eq!(guarded, None);
}

/// With no later stage and no later append to retry it, a domain refusal is
/// reported with its original stop.
#[test]
fn a_refusal_in_the_last_stage_is_reported_at_once() {
    let (outcome, _) = run(&[("a", 4)], true);
    assert_eq!(refused(&outcome[0]).as_deref(), Some("DerivedValueOutOfRange"));
}
