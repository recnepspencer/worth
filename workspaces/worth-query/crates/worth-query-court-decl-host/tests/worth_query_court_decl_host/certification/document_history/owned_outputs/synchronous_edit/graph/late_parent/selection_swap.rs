//! Two connections selecting one route are two selections (C2 r2 B1c, typed
//! in C4): a reopened derivation that selects the same child through the
//! other connection dropped the selection it derived, and refuses
//! `SupersededSelectionDropped`.
use super::selection::require_dropped;
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

/// `decision-input(a)` to `second(a)` on `$port`, selected only while
/// `first:a` is `$value`.
macro_rules! selector {
    ($name:ident, $connection:ident, $port:ty, $identity:literal, $value:literal) => {
        pub struct $name;
        impl ApplicationConnectionIdentity for $name {
            const IDENTITY: &'static str = $identity;
        }
        impl
            ApplicationOccurrenceConnectionBinding<
                Schema,
                decision_input::Feature,
                hierarchy::Child,
            > for $name
        {
        }
        type $connection = ApplicationConnectionRef<
            Schema,
            decision_input::Feature,
            decision_input::OutputPort,
            hierarchy::Child,
            $port,
            $name,
        >;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $name {
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
                row: &query::Row,
            ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(if row.value == $value {
                    vec![second::Demand("a".into())]
                } else {
                    Vec::new()
                })
            }
        }
        no_prerequisite!($name, second::Demand);
    };
}
selector!(
    AtSeven,
    AtSevenConnection,
    hierarchy::ChildInput,
    "owned-outputs.late-parent.swap-at-seven",
    7
);
selector!(
    AtEleven,
    AtElevenConnection,
    hierarchy::SeedInput,
    "owned-outputs.late-parent.swap-at-eleven",
    11
);

/// `decision-input(a)` to `decision-write(b)`, which writes m = b + 1.
pub struct Writer;
impl ApplicationConnectionIdentity for Writer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.swap-writer";
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

pub struct Swap;
impl ApplicationOccurrenceComponent<Schema> for Swap {
    type Connections = (AtSevenConnection, AtElevenConnection, WriterConnection);
}
pub type Tree = ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<Swap>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.swap-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.swap-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (`b = 8`) roots `decision-input(a)` (first:a = 2 + 5 = 7): the
/// `ChildInput` connection selects `second(a)`, and `decision-write(b)`
/// writes m = 9. The parent reopens and derives first:a = 2 + 9 = 11, where
/// the `SeedInput` connection selects `second(a)` instead. The route is the
/// same, the selection is not: the reopened derivation dropped the one it
/// derived, and the edit refuses `SupersededSelectionDropped`.
#[test]
fn a_reopened_derivation_selecting_the_same_route_through_another_connection_refuses() {
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
        .idempotency("late-parent-selection-swap".to_owned())
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
    require_dropped(outcome);
    assert_eq!(courts::native_head(&host), head);
}
