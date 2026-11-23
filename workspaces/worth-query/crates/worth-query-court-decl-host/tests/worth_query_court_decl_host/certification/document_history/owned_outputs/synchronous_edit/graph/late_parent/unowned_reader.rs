//! An independent producer is a publication but owns no scope: Native lets
//! no successor round continue its route (`next_producer_round`). A later
//! writer changing its decision read is left to final qualification, as for
//! an ordinary participant (PR C C3a r3 blocker 2).
use super::*;
use family::{decision_input, decision_write};

/// No supplier: model `b` has no members.
macro_rules! no_prerequisite {
    ($connection:ty, $demand:ty) => {
        impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for $connection {
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
                _: &$demand,
            ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::members::Read("b".into()))
            }
            fn parent_from_prerequisite(
                row: &query::Row,
            ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok(decision_input::Demand(row.key.clone()))
            }
        }
    };
}

/// `decision-input(a)` to `child(m)`: the independent member family, which
/// reads `m` through its source and creates `child:m`.
pub struct Reader;
impl ApplicationConnectionIdentity for Reader {
    const IDENTITY: &'static str = "owned-outputs.late-parent.unowned-reader";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, hierarchy::Child>
    for Reader
{
}
type ReaderConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    hierarchy::Child,
    hierarchy::ChildInput,
    Reader,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Reader {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = child::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![child::Demand("m".into())])
    }
}
no_prerequisite!(Reader, child::Demand);

/// `decision-input(a)` to `decision-write(b)`, which writes m = b + 1.
pub struct Writer;
impl ApplicationConnectionIdentity for Writer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.unowned-reader-writer";
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
no_prerequisite!(Writer, decision_write::Demand);

pub struct ReadThenWrite;
impl ApplicationOccurrenceComponent<Schema> for ReadThenWrite {
    type Connections = (ReaderConnection, WriterConnection);
}
pub type Tree =
    ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<ReadThenWrite>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.unowned-reader-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.unowned-reader-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// One stage (`b = 8`) enters `decision-input(a)` (first:a = 2 + 5), which
/// discovers `child(m)` (an independent publication reading m = 5) and then
/// `decision-write(b)`, which writes m = 9. The parent reopens and derives
/// again; `child(m)` cannot: Native continues only an owned route. Its
/// changed decision read stays with final qualification, which refuses
/// `CoupledReadSupportChanged` naming the later producer, exactly as for an
/// ordinary participant; the edit never reaches Native's owned-route
/// refusal.
#[test]
fn an_independent_publications_changed_decision_read_refuses_at_final_qualification() {
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
        .idempotency("late-parent-unowned-reader".to_owned())
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
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("a later writer must not commit over the independent reader: {outcome:?}")
    };
    let failure = denial
        .invariant_execution_failure()
        .expect("final candidate support refuses");
    assert_eq!(
        failure.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged,
        "{failure:?}"
    );
    assert!(failure.detail().contains("MemberValue"), "{failure:?}");
    assert!(
        failure.detail().contains("later writer producer"),
        "{failure:?}"
    );
}
