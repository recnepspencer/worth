//! One append changes a read that a parent and its already-done consumer
//! both made (PR C C3a r1 blocker 1). Qualification reopens both; the
//! consumer is reopened with its parent and again as its own reader, which
//! must raise its owed position, not abandon the edit.
use super::super::super::interleaved::component_carriage::parent as owner;
use super::*;
use family::{decision_input, decision_write, parity_input};

/// `decision-input(a)` to `parity(a)`, discovered from `first:a`, with
/// `decision-input(a)` as the child's strict prerequisite supplier. Both read
/// the member `m`.
pub struct Shared;
impl ApplicationConnectionIdentity for Shared {
    const IDENTITY: &'static str = "owned-outputs.late-parent.shared";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, parity_input::Feature>
    for Shared
{
}
type SharedConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    parity_input::Feature,
    parity_input::InputPort,
    Shared,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Shared {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = parity_input::Demand;
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
        Ok(vec![parity_input::Demand("a".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Shared {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &parity_input::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("first:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
/// `decision-input(a)` to `decision-write(b)`, discovered from `first:a`
/// after `parity(a)`; it names no prerequisite, so it consumes nothing.
pub struct Writer;
impl ApplicationConnectionIdentity for Writer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.shared-writer";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, decision_write::Feature>
    for Writer
{
}
type WriterConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Writer,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Writer {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
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
        // Model `b` has no members: the writer names no prerequisite supplier.
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
pub struct SharedSupply;
impl ApplicationOccurrenceComponent<Schema> for SharedSupply {
    type Connections = (SharedConnection, WriterConnection);
}
pub type Tree =
    ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<SharedSupply>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.shared-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.shared-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (`b = 8`) enters `decision-input(a)`, which derives first:a from
/// m = 5; from it `parity(a)` (its prerequisite consumer, also reading m)
/// and then `decision-write(b)` derive. The writer appends after both are
/// done and writes m = 9: one append changes both reads. Both re-derive and
/// the edit commits first:a = 2 + 9 = 11 and parity:a = 2 % 2 + 9 = 9.
#[test]
fn one_append_changing_a_parent_and_its_done_consumers_shared_read_reopens_both() {
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
        .idempotency("late-parent-shared-read".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    super::derived::committed(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let value = |key: &str| {
        request
            .query(query::first_output::Read(key.into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value
    };
    assert_eq!(value("first:a"), 11);
    assert_eq!(value("parity:a"), 9);
}
