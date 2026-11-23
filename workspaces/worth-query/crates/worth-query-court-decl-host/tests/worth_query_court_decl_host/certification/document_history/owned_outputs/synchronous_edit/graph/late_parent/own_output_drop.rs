//! A successor round that reaches a record its own chain deleted, through a
//! link served at the chain head (#277 step 1). The record is covered, so the
//! round reads its presence where it reads its fields: at the chain head,
//! where it exists.
use super::*;
use family::c3b::child_drop;
use family::decision_write;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-drop-start";
}
impl
    ApplicationOccurrenceConnectionBinding<
        Schema,
        super::super::super::super::program::Entry,
        child_drop::Feature,
    > for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    super::super::super::super::program::Entry,
    super::super::super::super::program::EntryOutput,
    child_drop::Feature,
    child_drop::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = child_drop::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(child_drop::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<child_drop::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-drop-discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, child_drop::Feature, decision_write::Feature>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    child_drop::Feature,
    child_drop::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = child_drop::Demand;
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
        Ok(vec![decision_write::Demand("a".into())])
    }
}
pub type Tree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-drop-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.own-drop-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// The first edit (`a = 8`) derives `first:a` with its child `kid:a`, and
/// commits: 8 + 9, one child. The second (`a = 9`) re-enters `child-drop(a)`
/// in its Preserve posture. The head round unlinks and deletes `kid:a`; its
/// child `decision-write(a)` sets m = 10, and `first:a` reopens as a successor
/// round of its own chain. That round reads `first:a`'s children through the
/// link served at the chain head, reaches `kid:a`, and reads its presence.
/// `kid:a` is a record the chain wrote, so its presence is served where its
/// fields are, at the chain head: it exists there. The round goes on to drop
/// the child again, a write Native cannot yet fold onto the chain's own
/// delete, so the edit stops typed and nothing publishes.
/// Red before step 1: untyped `MissingAuthoritativeFact`, the presence read at
/// the current image, where the chain's delete had already removed `kid:a`.
#[test]
fn a_successor_reading_the_presence_of_the_record_its_chain_deleted_refuses_typed() {
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
    let edit = |idempotency: &str, value| {
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
        (outcome, _execution.wave_events_for_test())
    };
    let (outcome, events) = edit("own-output-drop-first", 8);
    assert!(
        matches!(outcome, Outcome::Committed(_)),
        "the first edit commits first:a and kid:a: {outcome:?}; {events:#?}"
    );
    assert_eq!(first(&request), Some(8 + 9));
    assert_eq!(kid(&request), Some(0));
    let head = courts::native_head(&host);
    let (outcome, events) = edit("own-output-drop", 9);
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains(
            "CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }"
        ),
        "{detail}; {events:#?}"
    );
    assert!(
        events.iter().any(|event| event == "reopen task=0 owed=0"),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(first(&request), Some(8 + 9));
    assert_eq!(kid(&request), Some(0));
}

/// The value of `first:a` as a public read sees it.
fn first(request: &family::Request<'_, '_, '_>) -> Option<u64> {
    let result = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .ok()?;
    result.rows().first().map(|row| row.value)
}

/// The value of `kid:a` as a public read sees it, if the record exists.
fn kid(request: &family::Request<'_, '_, '_>) -> Option<u64> {
    let result = request
        .query(query::child_output::Read("kid:a".into()))
        .execute()
        .ok()?;
    result.rows().first().map(|row| row.value)
}
