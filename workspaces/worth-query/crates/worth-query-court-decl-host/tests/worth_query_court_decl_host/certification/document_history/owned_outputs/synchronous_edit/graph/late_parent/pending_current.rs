//! A pending domain refusal over a current-output lookup that found nothing
//! (PR C C4 r2 blocker 2): `current_output` returns `Missing`, a read no
//! fact key names. The refusal must still wait, so the sibling that publishes
//! `saturated:a` later in the same drive retries it.
use super::*;
use family::c3b::{guarded_current, saturated_input};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// The edit roots `first(a)`.
pub struct Root;
impl ApplicationConnectionIdentity for Root {
    const IDENTITY: &'static str = "owned-outputs.late-parent.current-root";
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
sibling!(
    Guard,
    GuardConnection,
    guarded_current::Feature,
    guarded_current::InputPort,
    guarded_current::Demand("a".into()),
    guarded_current::Demand,
    "owned-outputs.late-parent.current-guard"
);
sibling!(
    Write,
    WriteConnection,
    saturated_input::Feature,
    saturated_input::InputPort,
    saturated_input::Demand("a".into()),
    saturated_input::Demand,
    "owned-outputs.late-parent.current-sibling"
);
pub struct GuardThenWrite;
impl ApplicationOccurrenceComponent<Schema> for GuardThenWrite {
    type Connections = (GuardConnection, WriteConnection);
}
pub type Tree = ApplicationOutputGraph<RootConnection, ApplicationOutputOccurrences<GuardThenWrite>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.current-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.current-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage roots `first(a)`, which discovers, in order,
/// `guarded-current(a)` (refused while `saturated-input(a)` publishes no
/// current output for model `a`) and `saturated-input(a)`. The guard runs
/// first: `current_output` returns `Missing` and it refuses, waiting on every
/// append (that lookup has no fact key). The sibling then publishes
/// `saturated:a`, so the guard is retried in the same drive and derives
/// a + 1 = 3. Red before the fix: the lookup left no proof, nothing retried
/// the guard, and the edit refused at it.
#[test]
fn a_refusal_over_a_missing_current_output_is_retried_when_a_sibling_publishes_it() {
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
        .idempotency("pending-current".to_owned())
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
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert!(
        events.iter().any(|event| event.starts_with("pending refusal task=")),
        "the guard's refusal waited: {events:#?}"
    );
    let gm = request
        .query(query::first_output::Read("gc:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    assert_eq!(gm, 3);
}
