//! A pending domain refusal retried within one drive (PR C C4, §1.7): the
//! only shape where deferral is observable under base order, since a task
//! runs at a barrier only when it needs it and otherwise in the final drive.
use super::*;
use family::c3b::guarded_member;
use family::decision_write;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// The edit roots `first(a)`.
pub struct Root;
impl ApplicationConnectionIdentity for Root {
    const IDENTITY: &'static str = "owned-outputs.late-parent.retry-root";
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
    guarded_member::Feature,
    guarded_member::InputPort,
    guarded_member::Demand("a".into()),
    guarded_member::Demand,
    "owned-outputs.late-parent.retry-guard"
);
sibling!(
    Write,
    WriteConnection,
    decision_write::Feature,
    decision_write::InputPort,
    decision_write::Demand("b".into()),
    decision_write::Demand,
    "owned-outputs.late-parent.retry-writer"
);
pub struct GuardThenWrite;
impl ApplicationOccurrenceComponent<Schema> for GuardThenWrite {
    type Connections = (GuardConnection, WriteConnection);
}
pub type Tree = ApplicationOutputGraph<RootConnection, ApplicationOutputOccurrences<GuardThenWrite>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.retry-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.retry-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// (a) One stage (b = 7) roots `first(a)`, which discovers, in order,
/// `guarded-member(a)` (a + m, refused while m is odd) and `decision-write(b)`
/// (m = b + 1). The guard runs first over m = 5 and refuses; its refusal
/// waits on what it read. The writer then writes m = 8: that append marks the
/// refusal's proof (member m's value), so the guard is retried in the same
/// drive and derives 2 + 8 = 10. With deferral disabled the edit refuses at
/// the guard (red).
#[test]
fn a_refusal_a_later_append_in_the_same_drive_fixes_is_retried_and_commits() {
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
        .idempotency("pending-retry".to_owned())
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
        .query(query::first_output::Read("gm:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    assert_eq!(gm, 10);
}
