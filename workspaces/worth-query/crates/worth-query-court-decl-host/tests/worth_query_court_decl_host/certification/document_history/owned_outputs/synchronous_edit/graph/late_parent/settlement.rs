//! R3 unchanged-output settlement (PR C C3b): a done consumer of a reopened
//! derivation keeps its consumption when the replacement publishes exactly
//! what it read, and runs again only when the replacement differs.
use super::super::super::interleaved::component_carriage::parent as owner;
use super::*;
use family::{decision_write, saturated_input};
mod current_output;
mod cycle;
mod recorded;

/// The edit roots `saturated-input(a)`: a + min(m, 5).
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-start";
}
type Entry = super::super::super::super::program::Entry;
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, saturated_input::Feature> for Start {}
type StartConnection = ApplicationConnectionRef<
    Schema,
    Entry,
    super::super::super::super::program::EntryOutput,
    saturated_input::Feature,
    saturated_input::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = saturated_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(saturated_input::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<saturated_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}

/// `saturated-input(a)` to `second(a)`, which reads only model `a`, with the
/// saturated parent as its strict prerequisite supplier: it consumes it.
pub struct Consumer;
impl ApplicationConnectionIdentity for Consumer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-consumer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, saturated_input::Feature, hierarchy::Child>
    for Consumer
{
}
type ConsumerConnection = ApplicationConnectionRef<
    Schema,
    saturated_input::Feature,
    saturated_input::OutputPort,
    hierarchy::Child,
    hierarchy::ChildInput,
    Consumer,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Consumer {
    type RootDemand = saturated_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("saturated:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![second::Demand("a".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Consumer {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &second::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("saturated:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<saturated_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(saturated_input::Demand(row.key.clone()))
    }
}

/// `saturated-input(a)` to `decision-write(b)`, which writes m = b + 1; it
/// names no prerequisite, so it consumes nothing.
pub struct Writer;
impl ApplicationConnectionIdentity for Writer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-writer";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, saturated_input::Feature, decision_write::Feature>
    for Writer
{
}
type WriterConnection = ApplicationConnectionRef<
    Schema,
    saturated_input::Feature,
    saturated_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Writer,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Writer {
    type RootDemand = saturated_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("saturated:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![decision_write::Demand("b".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Writer {
    type Prerequisites = query::members::Read;
    type PrerequisiteRow = query::RowBinding;
    type Selection = ApplicationQueryResultRelationRef<
        query::members::Query,
        query::members::MembersSlot,
        Schema,
        ModelMember,
        Model,
        Member,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        query::members::members()
    }
    fn prerequisites_from_target(
        _: &decision_write::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        // Model `b` has no members: no prerequisite supplier.
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<saturated_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(saturated_input::Demand(row.key.clone()))
    }
}
pub struct ConsumeThenWrite;
impl ApplicationOccurrenceComponent<Schema> for ConsumeThenWrite {
    type Connections = (ConsumerConnection, WriterConnection);
}
pub type Tree =
    ApplicationOutputGraph<StartConnection, ApplicationOutputOccurrences<ConsumeThenWrite>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.settlement-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
fn first_output(request: &family::Request<'_, '_, '_>, key: &str) -> u64 {
    request
        .query(query::first_output::Read(key.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

/// One stage (`b = 8`) roots `saturated-input(a)` (2 + min(5, 5) = 7), which
/// discovers its consumer `second(a)` (2 + 1000) and then `decision-write(b)`,
/// which writes m = 9. The parent read m, so it reopens and derives again:
/// 2 + min(9, 5) = 7, exactly what its consumer read. The consumer settles
/// to the replacement and does not run again: the parent runs twice, the
/// consumer and the writer once each (4 producer contacts), and the edit
/// commits the sequential world (saturated:a = 7, second:a = 1002, m = 9).
#[test]
fn an_unchanged_replacement_settles_its_done_consumer_without_running_it_again() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-settlement".to_owned())
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
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert_eq!(first_output(&request, "saturated:a"), 7);
    assert_eq!(
        request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1002
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        4,
        "the parent twice, the settled consumer and the writer once: {events:#?}"
    );
    assert!(
        events.iter().any(|event| event.starts_with("settle consumer")),
        "{events:#?}"
    );
}
