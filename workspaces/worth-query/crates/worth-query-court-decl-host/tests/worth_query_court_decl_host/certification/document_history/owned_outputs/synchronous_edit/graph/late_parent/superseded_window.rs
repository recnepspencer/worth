//! R4 windows over a chain's superseded rounds (PR C C3a r2 blocker 1). A
//! successor replaces its chain's writer in place, but the image its earlier
//! round produced stays where it applied: a retained read that round changed
//! before the read's recorded invalidator changed inside the read's window.
//! Since C4 a selection's support is compared at every append, the superseded
//! round's own included, so such a change ends the window there and the
//! selection runs again (#238); the window check stays the backstop.
use super::super::super::interleaved::component_carriage::parent as owner;
use super::*;
use family::{decision_input, decision_write, ordinary_input};

/// `decision-input(a)` to `second(a)`, discovered from `first:a`, with
/// `decision-input(a)` as the child's strict prerequisite supplier: the
/// child consumes the parent and reopens with it (R3).
pub struct Consumer;
impl ApplicationConnectionIdentity for Consumer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.superseded-window-consumer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, hierarchy::Child>
    for Consumer
{
}
type ConsumerConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    hierarchy::Child,
    hierarchy::ChildInput,
    Consumer,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Consumer {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
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
        Ok(owner::Read("first:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}

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

/// `decision-input(a)` also discovers through `first:a` joined to its
/// model's `second:*` outputs (their values summed: 0 while none exists).
/// It selects nothing.
pub struct Watch;
impl ApplicationConnectionIdentity for Watch {
    const IDENTITY: &'static str = "owned-outputs.late-parent.superseded-window-watch";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, ordinary_input::Feature>
    for Watch
{
}
type WatchConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    ordinary_input::Feature,
    ordinary_input::InputPort,
    Watch,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Watch {
    type RootDemand = decision_input::Demand;
    type Discovery = query::joined::Read;
    type Demand = ordinary_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::joined::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Vec::new())
    }
}
no_prerequisite!(Watch, ordinary_input::Demand);

/// `decision-input(a)` to `decision-write(b)`, which writes m = b + 1.
pub struct Writer;
impl ApplicationConnectionIdentity for Writer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.superseded-window-writer";
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

pub struct Watched;
impl ApplicationOccurrenceComponent<Schema> for Watched {
    type Connections = (ConsumerConnection, WriterConnection);
}
/// The watch is a static edge: its discovery support is retained as a
/// query read of the parent's task, windowed when the parent reopens.
pub type Tree = ApplicationOutputGraph<
    program::Connection,
    (
        ApplicationOutputOccurrences<Watched>,
        ApplicationOutputEdge<WatchConnection, ApplicationOutputLeaf>,
    ),
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.superseded-window-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.superseded-window-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// The same graph without the watch, which selects nothing in every state:
/// committing the stage through it builds the world the edit must equal.
pub type BareTree =
    ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<Watched>>;
pub struct BareDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for BareDelivery {
    type Mode = ApplicationFixedDeliveryMode<BareTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.superseded-window-bare-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for BareDelivery {
    type Root = BareTree;
    type Projection = ApplicationNoProjection;
}
pub struct BareProgram;
impl ApplicationProgramDefinition<Schema> for BareProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<BareTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.superseded-window-bare-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<BareDelivery>();
        specs
    }
}

use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// first:a and second:a as committed.
fn committed(request: &family::Request<'_, '_, '_>) -> (u64, u64) {
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    let second = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    (first, second)
}

/// The stage `b = 8` committed through the graph without the watch.
fn sequential() -> (u64, u64) {
    let host = world::install_program_with_other_model::<BareProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-superseded-window-bare".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<BareTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    committed(&request)
}

/// One stage (`b = 8`) enters `decision-input(a)` (first:a = 2 + 5 at 1).
/// At 1 its occurrence component discovers `second(a)`, its prerequisite
/// consumer, and `decision-write(b)`; its static watch discovers nothing
/// through the join (no `second:a`: 0) and keeps that query read. `second(a)`
/// derives at 2 and creates `second:a` (2 + 1000), changing the watch's read.
/// Qualified use compares that selection support at the append at 2 (#238,
/// PR C C4): it changed, so its window ends at 2, authenticated by that exact
/// comparison, and the parent selects its children again (nothing, through a
/// join that now sums 1002). The writer appends at 3 and writes m = 9,
/// reopening the parent, which derives first:a = 2 + 9 = 11 and selects
/// again; `second(a)` reopens with it and derives 1002 again. Every read is
/// checked over its window, so the edit commits the single-stage fixed point:
/// first:a = 11, second:a = 1002, the world the same stage commits through
/// the graph without the watch. (Before C4 this refused at final support:
/// the window over the superseded round at 2 saw the change, and discovery
/// support had no owner to select again.)
#[test]
fn a_superseded_round_changing_a_watched_selection_selects_again() {
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
        .idempotency("late-parent-superseded-window".to_owned())
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
    assert!(
        events.iter().any(|event| event.starts_with("reselect children task=0")),
        "the changed watch selects again: {events:#?}"
    );
    // The watch selects nothing in every state, so the world equals the
    // stage committed without it: first:a = 2 + 9, second:a = 2 + 1000.
    assert_eq!(committed(&request), (11, 1002));
    assert_eq!(sequential(), (11, 1002));
}
