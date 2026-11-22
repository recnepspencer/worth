//! A dropped selection whose derived work is not a direct, done parent (PR
//! C C4 r5): a parent reopened since it derived, and a never-derived
//! intermediate whose own prerequisite derived. Either drop must refuse
//! `SupersededSelectionDropped`, never commit the derived work as an orphan.
use super::reopen_closed::{
    GuardConnection, RootConnection, SecondEdgeConnection, WriteEdgeConnection, SECOND_WRITER_A,
};
use super::*;
use super::super::super::interleaved::valued_members;
use std::sync::atomic::Ordering;

/// `first(a)` to `first(c)`, whose own incoming prerequisites are model
/// `a`'s members valued 8: `m` selects `first(b)`, any other `first(d)`.
pub struct ChildEdge;
impl ApplicationConnectionIdentity for ChildEdge {
    const IDENTITY: &'static str = "owned-outputs.late-parent.dropped-child";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent> for ChildEdge {}
type ChildEdgeConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Parent,
    hierarchy::ContinuationInput,
    ChildEdge,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for ChildEdge {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.key == "first:a" { vec![first::Demand("c".into())] } else { Vec::new() })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for ChildEdge {
    type Prerequisites = valued_members::Read;
    type PrerequisiteRow = valued_members::ChildBinding;
    type Selection = valued_members::ValuedMembers;
    fn prerequisite_rows() -> Self::Selection {
        valued_members::valued()
    }
    fn prerequisites_from_target(
        target: &first::Demand,
    ) -> Result<valued_members::Read, WorthQueryRequiredOutputConnectionDenial> {
        // Only `first(c)` selects; model `b` owns no member.
        Ok(valued_members::Read(if target.0 == "c" { "a" } else { "b" }.into()))
    }
    fn parent_from_prerequisite(
        row: &String,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(if row == "m" { "b" } else { "d" }.into()))
    }
}
/// The guard, both writers and the child edge in one occurrence component.
pub struct Component;
impl ApplicationOccurrenceComponent<Schema> for Component {
    type Connections = (
        GuardConnection,
        WriteEdgeConnection,
        SecondEdgeConnection,
        ChildEdgeConnection,
    );
}
pub type Tree = ApplicationOutputGraph<RootConnection, ApplicationOutputOccurrences<Component>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.dropped-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.dropped-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (a = 2) over members `m` (5) and `n` (`n`); `decision-write(b)`
/// sets m = 8 and `decision-write-n(a)` sets n = 3. Returns the outcome, the
/// wave events, and which outputs the candidate holds afterwards.
fn run(n: u64, key: &str) -> (String, Vec<String>, [bool; 3]) {
    SECOND_WRITER_A.store(true, Ordering::Relaxed);
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
        world::Configuration {
            second_member: Some(n),
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency(key.to_owned())
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
    let held = ["first:b", "first:c", "first:d"].map(|key| {
        request
            .query(query::first_output::Read(key.into()))
            .execute()
            .is_ok_and(|rows| !rows.rows().is_empty())
    });
    (format!("{outcome:?}"), events, held)
}

/// A never-derived intermediate (PR C C4 r5, blocker 2): unreachable as an
/// orphan. A task has prerequisites of its own only as an occurrence of a
/// prerequisite connection, so it is also a discovered child of a derived
/// parent and stays needed when another owner drops it. The guard closes on
/// `first(c)` (row `n` = 8); `first(c)`, discovered from `first(a)`, closes
/// on its own rows. `decision-write(b)` sets m = 8 and `decision-write-n(a)`
/// sets n = 3: the guard drops `first(c)`, never derived, and `first(c)`
/// re-closes on `{first(b)}`. It is still `first(a)`'s child, so it derives,
/// and the edit commits what one commit per stage commits for this one
/// stage: the final rows select `first(b)` (m = 8) and never `first(d)`.
#[test]
fn a_never_derived_intermediate_another_owner_drops_stays_needed_and_commits() {
    let (outcome, events, held) = run(8, "dropped-intermediate");
    assert!(outcome.starts_with("Committed"), "{outcome}; {events:#?}");
    assert_eq!(held, [true, true, false], "first:b, first:c, never first:d: {events:#?}");
}
