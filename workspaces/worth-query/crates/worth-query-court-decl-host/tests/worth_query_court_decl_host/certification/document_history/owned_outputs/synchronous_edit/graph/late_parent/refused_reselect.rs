//! A pending refusal an incoming prerequisite selection reopens (PR C C4 r2
//! blocker 4): only the guard's own prerequisite selector sees the change,
//! none of its refusal's proofs do, so only `reselect` can retry it.
use super::*;
use family::c3b::guarded_first;
use family::decision_write;
use super::super::super::interleaved::valued_members;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// The edit roots `first(a)`.
pub struct Root;
impl ApplicationConnectionIdentity for Root {
    const IDENTITY: &'static str = "owned-outputs.late-parent.reselect-root";
}
type Entry = super::super::super::super::program::Entry;
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, hierarchy::Parent> for Root {}
type RootConnection = ApplicationConnectionRef<
    Schema,
    Entry,
    super::super::super::super::program::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Root,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Root {
    type Source = edit::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(_: &edit::Intent) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Root> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Root as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}

/// `first(a)` to `$feature`'s `$demand`, discovered from `first:a`, with no
/// prerequisite supplier (model `b` has no members).
macro_rules! sibling {
    ($name:ident, $connection:ident, $feature:ty, $port:ty, $demand:expr, $demand_ty:ty, $identity:literal) => {
        pub struct $name;
        impl ApplicationConnectionIdentity for $name {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, $feature> for $name {}
        type $connection = ApplicationConnectionRef<
            Schema,
            hierarchy::Parent,
            hierarchy::Output,
            $feature,
            $port,
            $name,
        >;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $name {
            type RootDemand = first::Demand;
            type Discovery = query::first_output::Read;
            type Demand = $demand_ty;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_root(
                _: &Self::RootDemand,
            ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::first_output::Read("first:a".into()))
            }
            fn demands_from_discovery(
                _: &query::Row,
            ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(vec![$demand])
            }
        }
        impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for $name {
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
                _: &$demand_ty,
            ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::members::Read("b".into()))
            }
            fn parent_from_prerequisite(
                row: &query::Row,
            ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok(first::Demand(row.key.clone()))
            }
        }
    };
}
/// `first(a)` to `guarded-first(a)`, whose incoming prerequisites are model
/// `a`'s members valued 8, each selecting `first(b)`.
pub struct Guard;
impl ApplicationConnectionIdentity for Guard {
    const IDENTITY: &'static str = "owned-outputs.late-parent.reselect-guard";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, guarded_first::Feature> for Guard {}
type GuardConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    guarded_first::Feature,
    guarded_first::InputPort,
    Guard,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Guard {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = guarded_first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![guarded_first::Demand("a".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Guard {
    type Prerequisites = valued_members::Read;
    type PrerequisiteRow = valued_members::ChildBinding;
    type Selection = valued_members::ValuedMembers;
    fn prerequisite_rows() -> Self::Selection {
        valued_members::valued()
    }
    fn prerequisites_from_target(
        _: &guarded_first::Demand,
    ) -> Result<valued_members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(valued_members::Read("a".into()))
    }
    fn parent_from_prerequisite(
        _: &String,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("b".into()))
    }
}
sibling!(
    Write,
    WriteConnection,
    decision_write::Feature,
    decision_write::InputPort,
    decision_write::Demand("b".into()),
    decision_write::Demand,
    "owned-outputs.late-parent.reselect-writer"
);
pub struct GuardThenWrite;
impl ApplicationOccurrenceComponent<Schema> for GuardThenWrite {
    type Connections = (GuardConnection, WriteConnection);
}
pub type Tree = ApplicationOutputGraph<RootConnection, ApplicationOutputOccurrences<GuardThenWrite>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.reselect-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.reselect-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (b = 7) roots `first(a)`, which discovers, in order,
/// `guarded-first(a)` (refused while `first:b` is absent) and
/// `decision-write(b)` (m = b + 1). The guard closes with no prerequisite
/// (no member of `a` is valued 8 while m is 5), finds no `first:b` and
/// refuses. The writer sets m = 8: none of the guard's proofs changes, but its
/// incoming selection now selects `first(b)`. `reselect` reopens the refused
/// guard, which closes on `first(b)`, sees `first:b` and derives a + 1 = 3.
/// Red before the fix: `reselect` ignored a refused task, nothing retried
/// it, and the edit refused at it.
#[test]
fn a_refused_task_whose_incoming_selection_changes_is_reopened_and_commits() {
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
        .idempotency("refused-reselect".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "a".into(),
        value: 2,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert!(
        events.iter().any(|event| event.starts_with("pending refusal task=")),
        "the guard's refusal waited: {events:#?}"
    );
    let gf = request
        .query(query::first_output::Read("gf:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    assert_eq!(gf, 3);
}
