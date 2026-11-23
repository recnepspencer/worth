//! A successor round that reaches a record its own chain created through a
//! relation read at the current image (PR C C4, #277). Under PR C alone the
//! own-output view was entity-only, so the round stopped typed
//! (`Postponed { 119, PriorEffectRead }`). #277 serves relations through the
//! view too: the chain's own `FirstOf` link is read as it was at the chain
//! head's pre-pin, where it does not exist, so the round re-derives without
//! reaching the record and the edit commits the sequential world.
use super::*;
use family::c3b::self_relation;
use family::decision_write;
thread_local! {
    /// The model `decision-write` derives from (per court, on its thread).
    /// With `a`, one stage on `a` re-derives `self-relation(a)` as the head
    /// round and reopens it through member `m`.
    static CHILD_SOURCE: std::cell::Cell<&'static str> = const { std::cell::Cell::new("b") };
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-gap-start";
}
impl
    ApplicationOccurrenceConnectionBinding<
        Schema,
        super::super::super::super::program::Entry,
        self_relation::Feature,
    > for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    super::super::super::super::program::Entry,
    super::super::super::super::program::EntryOutput,
    self_relation::Feature,
    self_relation::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = self_relation::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(self_relation::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<self_relation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-gap-discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, self_relation::Feature, decision_write::Feature>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    self_relation::Feature,
    self_relation::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = self_relation::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        assert_eq!(row.key.as_str(), "first:a");
        Ok(vec![decision_write::Demand(
            CHILD_SOURCE.with(std::cell::Cell::get).into(),
        )])
    }
}
pub type Tree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-gap-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.own-gap-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (`b = 8`) enters `self-relation(a)`, which derives `first:a`
/// (2 + 5) and reads every output model `a` owns through `FirstOf`: none
/// yet. Its child `decision-write(b)` sets m = 9, so `first:a` reopens as a
/// successor round of its own chain. That round's `FirstOf` read at the
/// current image would reach `first:a`, which the chain created and linked;
/// served through the own-output view (#277), the chain's link is read at the
/// head pre-pin, where it does not exist, so the round reads no output and
/// re-derives `first:a` from m = 9. The edit commits the sequential world:
/// one commit per stage reads m = 9 on demand, and no output yet (2a/2b).
/// Red before #277: `Postponed { 119, PriorEffectRead }` under PR C's guard.
#[test]
fn a_successor_reaching_its_own_created_record_by_relation_commits_the_sequential_world() {
    own_output_relation_commits("own-output-gap");
}

/// The same edit with a traversal only (PR C C4 r7, #277 court 7b shape): the
/// successor's `FirstOf` read never reads the reached record's value. Under
/// PR C the chain-created endpoint denied the traversal
/// (`NativeRead::endpoint`) and the round stopped typed; #277 hides the
/// chain's own link, so nothing is reached and the edit commits the
/// sequential world. A link another writer made to the chain's record would
/// still be served and refuse typed (`coherent_endpoints`).
#[test]
fn a_successor_traversing_to_its_own_created_record_commits_the_sequential_world() {
    family::c3b::self_relation::TRAVERSE_ONLY.with(|flag| flag.set(true));
    own_output_relation_commits("own-output-traverse");
}

/// The same edit with Initial finding its prior output through its output
/// role instead of a `FirstKey` selection (#277 r4 A). The successor's create
/// of `first:a` writes the unique `FirstKey` it never selected, so only the
/// intrinsic unique-write dependency observes it. At the current image its
/// one holder is `first:a` itself, which the chain's round 1 created and
/// which Native's fold rewrites in place under the same key. Through the view
/// the value is free, and the edit commits the sequential world.
/// Red before r4 A: untyped `UniqueValueTaken` (`ProducerUnavailable`).
#[test]
fn a_successor_recreating_its_unique_output_commits_the_sequential_world() {
    family::c3b::self_relation::BLIND_PRIOR.with(|flag| flag.set(true));
    own_output_relation_commits("own-output-unique");
}

/// The preserved-entity variant (#277 court 7), with `decision-write` on
/// model `a`. The first edit (`a = 8`) commits `first:a` = 8 + 9. The second
/// (`a = 9`) re-enters `self-relation(a)` in its Preserve posture: the head
/// round writes `first:a` = 9 + 9, its child sets m = 10, and `first:a`
/// reopens as a successor round of its own chain. The successor's `FirstOf`
/// read reaches `first:a` through a link that predates the chain (uncovered),
/// and the record exists at the chain head: its value is read there, the view
/// is coherent, and the edit commits the sequential world, `first:a` = 9 + 10.
#[test]
fn a_successor_reaching_its_preserved_record_commits_the_sequential_world() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    CHILD_SOURCE.with(|source| source.set("a"));
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for (idempotency, value) in [
        ("own-output-preserved-first", 8),
        ("own-output-preserved", 9),
    ] {
        let head = courts::native_head(&host);
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let mut edit = request
            .edit_envelope()
            .idempotency(idempotency.to_owned())
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<Tree, _>(edit::Intent {
            key: "a".into(),
            value,
        })
        .unwrap();
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let events = _execution.wave_events_for_test();
        assert!(
            matches!(outcome, Outcome::Committed(_)),
            "{outcome:?}; {events:#?}"
        );
        assert!(
            events.iter().any(|event| event == "reopen task=0 owed=0"),
            "{events:#?}"
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 3,
            "the head round, its child, and the successor round each derive"
        );
        assert_eq!(
            request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            value + value + 1
        );
        assert_ne!(courts::native_head(&host), head);
    }
}

fn own_output_relation_commits(idempotency: &str) {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency(idempotency.to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    assert!(
        matches!(outcome, Outcome::Committed(_)),
        "{outcome:?}; {events:#?}"
    );
    assert!(
        events
            .iter()
            .any(|event| event.starts_with("reopen task=0")),
        "first:a re-derives as a successor round: {events:#?}"
    );
    assert_rederived_first(&request);
    assert_ne!(courts::native_head(&host), head);
}

/// The same edit, the successor reading model `a`'s outputs' models through
/// the derived `FirstSiblings` (#277 court 8, A6; deviation §1). Native's
/// derived traversal reads the current image, not the own-output view, and
/// its probe of `a`'s incoming `FirstOf` list crosses the chain's own link to
/// `first:a`. That answer holds the chain's effect, so the view is marked
/// incoherent at `a` (an anchor the chain never wrote) and the traversal
/// denies; the round stops typed and nothing publishes. It never commits on
/// the unmerged answer and never ends untyped.
#[test]
fn a_successor_derived_traversal_over_its_own_link_refuses_typed() {
    family::c3b::self_relation::DERIVED_READ.with(|flag| flag.set(true));
    own_output_relation_refuses_typed("own-output-derived");
}

/// As above, but the handler ignores the derived read's denial and goes on
/// to decide. The failed derived read is recorded before any decision fact,
/// so the decision cannot seal: the round still stops typed.
#[test]
fn a_successor_swallowing_its_derived_traversal_denial_still_refuses_typed() {
    family::c3b::self_relation::DERIVED_READ.with(|flag| flag.set(true));
    family::c3b::self_relation::SWALLOW_DERIVED.with(|flag| flag.set(true));
    own_output_relation_refuses_typed("own-output-derived-swallowed");
}

fn own_output_relation_refuses_typed(idempotency: &str) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency(idempotency.to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains(
            "CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }"
        ),
        "{detail}; {events:#?}"
    );
    // The stop names anchor `a`, which the chain never wrote: the derived
    // guard caught the crossing, not a later fact on the chain's own record.
    let anchor = request
        .query(query::producer_model::Read::from("a".to_owned()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    assert!(
        detail.contains(&format!("affected_records: [Entity({anchor:?})]")),
        "{detail}"
    );
    assert_eq!(courts::native_head(&host), head);
}
