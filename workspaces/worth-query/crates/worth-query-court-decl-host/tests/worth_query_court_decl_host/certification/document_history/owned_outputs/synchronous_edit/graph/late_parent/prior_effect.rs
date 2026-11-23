//! Successor independence on an entity read (PR C C3b, plan §5
//! "`PriorEffectRead`: a successor that reads a non-output key P1 wrote"): a
//! successor round whose source query reads a record its own chain wrote
//! observes that round's effect, which the own-output view does not serve.
use super::*;
use family::{decision_input, self_write};

/// `decision-input(a)` to `self-write(m)`, discovered from `first:a`; it
/// names no prerequisite supplier.
pub struct SelfWriter;
impl ApplicationConnectionIdentity for SelfWriter {
    const IDENTITY: &'static str = "owned-outputs.late-parent.self-writer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, self_write::Feature>
    for SelfWriter
{
}
type SelfWriterConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    self_write::Feature,
    self_write::InputPort,
    SelfWriter,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for SelfWriter {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = self_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![self_write::Demand("m".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for SelfWriter {
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
        _: &self_write::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        // Model `b` has no members: no prerequisite supplier.
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
pub struct SelfWrites;
impl ApplicationOccurrenceComponent<Schema> for SelfWrites {
    type Connections = (SelfWriterConnection,);
}
pub type Tree = ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<SelfWrites>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.self-writer-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.self-writer-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// One stage (`b = 7`, unchanged) roots `decision-input(a)`: first:a = 2 + 5
/// at 1. It discovers `self-write(m)`, which reads first:a (7) and its source
/// member m (5) through the public member query and writes m = 5 + 1 = 6 at
/// 2. That reopens `decision-input(a)` (first:a = 2 + 6 = 8 at 3), whose new
/// output reopens `self-write(m)`. Its successor round reads m through the
/// source query again: 5 at the chain head, 6 now, a key its own head wrote
/// that the own-output view does not serve. It stops before append with
/// `Postponed{119, PriorEffectRead}` and the edit publishes nothing.
#[test]
fn a_successor_reading_its_own_write_through_its_source_query_stops() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-prior-effect".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 7,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("the successor observed its own round's write: {outcome:?}; {events:#?}")
    };
    let detail = format!("{denial:?}");
    assert!(
        detail.contains("CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }"),
        "{detail}; {events:#?}"
    );
    assert!(request
        .query(query::first_output::Read("self:m".into()))
        .execute()
        .is_err());
}
