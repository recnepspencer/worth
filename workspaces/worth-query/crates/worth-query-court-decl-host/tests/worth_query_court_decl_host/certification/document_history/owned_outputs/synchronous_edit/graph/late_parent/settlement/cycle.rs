//! A consumption cycle that R3 settlement would form (PR C C3b r2): a
//! reopened parent's later round reads its own settled `current_output`
//! consumer. No order of the group's members derives both, so the edit
//! refuses whole before the World commit.
use super::*;
use family::c3b::{cycle_consumer, cycle_parent};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryInvariantExecutionDenialKind,
};

/// The edit roots `cycle-parent(a)`.
pub struct CycleStart;
impl ApplicationConnectionIdentity for CycleStart {
    const IDENTITY: &'static str = "owned-outputs.late-parent.cycle-start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, cycle_parent::Feature> for CycleStart {}
type CycleStartConnection = ApplicationConnectionRef<
    Schema,
    Entry,
    super::super::super::super::super::program::EntryOutput,
    cycle_parent::Feature,
    cycle_parent::InputPort,
    CycleStart,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for CycleStart {
    type Source = edit::Binding;
    type Demand = cycle_parent::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(cycle_parent::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, CycleStart> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<cycle_parent::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <CycleStart as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}

/// `cycle-parent(a)` to `cycle-consumer(c)`, which reads `cp:a` through
/// `current_output`, with the parent as its strict prerequisite supplier.
pub struct CycleReader;
impl ApplicationConnectionIdentity for CycleReader {
    const IDENTITY: &'static str = "owned-outputs.late-parent.cycle-consumer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, cycle_parent::Feature, cycle_consumer::Feature>
    for CycleReader
{
}
type CycleReaderConnection = ApplicationConnectionRef<
    Schema,
    cycle_parent::Feature,
    cycle_parent::OutputPort,
    cycle_consumer::Feature,
    cycle_consumer::InputPort,
    CycleReader,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for CycleReader {
    type RootDemand = cycle_parent::Demand;
    type Discovery = query::first_output::Read;
    type Demand = cycle_consumer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("cp:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![cycle_consumer::Demand("c".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for CycleReader {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &cycle_consumer::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("cp:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<cycle_parent::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(cycle_parent::Demand(row.key.clone()))
    }
}

/// `cycle-parent(a)` to `decision-write(b)`, which writes m = b + 1 and
/// consumes nothing.
pub struct CycleWriter;
impl ApplicationConnectionIdentity for CycleWriter {
    const IDENTITY: &'static str = "owned-outputs.late-parent.cycle-writer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, cycle_parent::Feature, decision_write::Feature>
    for CycleWriter
{
}
type CycleWriterConnection = ApplicationConnectionRef<
    Schema,
    cycle_parent::Feature,
    cycle_parent::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    CycleWriter,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for CycleWriter {
    type RootDemand = cycle_parent::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("cp:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![decision_write::Demand("b".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for CycleWriter {
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
    ) -> Result<cycle_parent::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(cycle_parent::Demand(row.key.clone()))
    }
}
pub struct CycleReadThenWrite;
impl ApplicationOccurrenceComponent<Schema> for CycleReadThenWrite {
    type Connections = (CycleReaderConnection, CycleWriterConnection);
}
pub type CycleTree =
    ApplicationOutputGraph<CycleStartConnection, ApplicationOutputOccurrences<CycleReadThenWrite>>;
pub struct CycleDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for CycleDelivery {
    type Mode = ApplicationFixedDeliveryMode<CycleTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.cycle-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for CycleDelivery {
    type Root = CycleTree;
    type Projection = ApplicationNoProjection;
}
pub struct CycleProgram;
impl ApplicationProgramDefinition<Schema> for CycleProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<CycleTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.cycle-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::super::program::entry_spec::<CycleDelivery>();
        specs
    }
}

/// The default world (a = 2, b = 7, m = 5) plus model `c` (3).
fn install() -> WorthQueryProgramApplicationRuntime<Schema, CycleProgram> {
    let declaration = Schema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, CycleProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005357").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| {
            world::seed(graph, installed, 2, Some(7))?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Model::reference(),
                    WorthQueryApplicationEntityKey::new("c").unwrap(),
                )
                .field(ModelKey::at("ModelKey"), "c".to_owned())
                .field(ModelValue::at("ModelValue"), 3u64),
            )
        },
    )
    .unwrap()
}

/// One stage (`b = 8`) roots `cycle-parent(a)`. Round 1 reads m = 5 and
/// derives 2 + min(5, 5) = 7 without reading its consumer.
/// `cycle-consumer(c)` reads cp:a = 7 through `current_output` (Appended),
/// and `decision-write(b)` writes m = 9. The parent reopens: round 2 reads
/// m = 9 > 5, so it reads the done consumer's cyc:c = 7 through
/// `current_output`, and derives 7 again. The consumer settles to round 2,
/// which consumed the consumer: no order of the members derives both.
/// Final support refuses the edit whole before the World commit with
/// `CoupledConsumptionCycle`, attributed to both, and nothing publishes.
#[test]
fn a_parent_round_that_reads_its_settled_consumer_refuses_before_commit() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("settlement-cycle".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<CycleTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("the consumption cycle refuses before the World commit: {outcome:?}; {events:#?}")
    };
    let failure = denial
        .invariant_execution_failure()
        .unwrap_or_else(|| panic!("final candidate support refuses: {denial:?}"));
    assert_eq!(
        failure.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledConsumptionCycle,
        "{denial:?}"
    );
    // Both the consumer and the parent round it settled to are on the cycle.
    assert_eq!(failure.affected_records().len(), 2, "{denial:?}");
    assert!(
        events.iter().any(|event| event.starts_with("settle consumer")),
        "{denial:?}; {events:#?}"
    );
    // Nothing publishes: neither output exists to resolve.
    for key in ["cp:a", "cyc:c"] {
        let read = request.query(query::first_output::Read(key.into())).execute();
        let refusal = read.err().map(|cause| format!("{cause:?}"));
        assert!(
            refusal.as_ref().is_some_and(|cause| cause.contains("UnknownEntity")),
            "{key} publishes nothing: {refusal:?}"
        );
    }
}
