//! Repeated R3 settlement (PR C C3b r1): a parent reopened by every write the
//! converging cross-write makes to its member, with an identical output
//! each time, keeps its `current_output` consumer settled to its latest
//! round.
use super::*;
use family::{capped_consumer, capped_reader};

/// `decision-input(a)` to `capped-reader(r)`, which reads `x:r` capped at 1.
pub struct Capped;
impl ApplicationConnectionIdentity for Capped {
    const IDENTITY: &'static str = "owned-outputs.late-parent.capped-reader";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, capped_reader::Feature>
    for Capped
{
}
type CappedConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    capped_reader::Feature,
    capped_reader::InputPort,
    Capped,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Capped {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = capped_reader::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![capped_reader::Demand("r".into())])
    }
}

/// `capped-reader(p)` to `capped-consumer(p)`, which reads `capped:p` through
/// `current_output`, with the capped reader as its strict prerequisite.
pub struct CappedReading;
impl ApplicationConnectionIdentity for CappedReading {
    const IDENTITY: &'static str = "owned-outputs.late-parent.capped-consumer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, capped_reader::Feature, capped_consumer::Feature>
    for CappedReading
{
}
type CappedReadingConnection = ApplicationConnectionRef<
    Schema,
    capped_reader::Feature,
    capped_reader::OutputPort,
    capped_consumer::Feature,
    capped_consumer::InputPort,
    CappedReading,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for CappedReading {
    type RootDemand = capped_reader::Demand;
    type Discovery = query::first_output::Read;
    type Demand = capped_consumer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &capped_reader::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("capped:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![capped_consumer::Demand(
            row.key.strip_prefix("capped:").unwrap().into(),
        )])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for CappedReading {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        target: &capped_consumer::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read(format!("capped:{}", target.0)))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<capped_reader::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(capped_reader::Demand(row.key.clone()))
    }
}
pub struct CappedConsumers;
impl ApplicationOccurrenceComponent<Schema> for CappedConsumers {
    type Connections = (CappedReadingConnection,);
}
pub type Tree = ApplicationOutputGraph<
    program::Connection,
    (
        // First, so the capped reader precedes the cross readers in a batch
        // and runs at every writer round.
        ApplicationOutputEdge<CappedConnection, ApplicationOutputOccurrences<CappedConsumers>>,
        ApplicationOutputEdge<
            ReadersConnection,
            ApplicationOutputOccurrences<converging::Writers>,
        >,
    ),
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.capped-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.capped-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// `capped-reader(r)` derives min(x:r, 1) = 1 and its `current_output`
/// consumer `capped-consumer(r)` cc:r = 1. The converging cross-write's
/// writer `cross-converge(p)` moves x:r from 1 to min(1 + 1, 3) = 2 and, after
/// `cross-converge(r)` moves x:p to 2, to min(2 + 1, 3) = 3. Each move
/// reopens the capped reader, whose rounds 2 and 3 derive 1 again; the
/// consumer settles to round 2 and then to round 3, its latest, which final
/// support authenticates. The edit commits the fixed point (3s) with
/// capped:r = cc:r = 1, the consumer running once.
#[test]
fn a_current_output_consumer_settles_again_to_each_later_identical_round() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install::<Program>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let outcome: Outcome = cross_edit!(&host, request, Tree);
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    for key in ["cross:p", "cross:r", "converge:p", "converge:r"] {
        assert_eq!(first_output(&request, key), 3, "{key}");
    }
    assert_eq!(first_output(&request, "capped:r"), 1);
    assert_eq!(first_output(&request, "cc:r"), 1);
    let settled = events
        .iter()
        .filter(|event| event.starts_with("settle consumer"))
        .count();
    assert_eq!(settled, 2, "settled at the second and third rounds: {events:#?}");
    let consumer_runs = events
        .iter()
        .filter(|event| event.starts_with("prepare") && event.contains("family=capped-consumer"))
        .count();
    assert_eq!(consumer_runs, 1, "{events:#?}");
}
