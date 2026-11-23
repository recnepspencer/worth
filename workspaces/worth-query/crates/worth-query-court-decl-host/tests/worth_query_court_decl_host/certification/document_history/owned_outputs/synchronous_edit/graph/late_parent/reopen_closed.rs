//! A second incoming-selection change while a reopened guard waits closed on
//! its new prerequisites (PR C C4 r2 blocker 1): it must re-close on the
//! current selection, never prepare on the selection it closed on.
use super::*;
use family::c3b::{decision_write_n, guarded_first};
use family::decision_write;
thread_local! {
    /// Whether `decision-write-n` reads model `a` (n = 3) instead of `b` (n =
    /// 8): per court, on the court's own thread.
    pub(super) static SECOND_WRITER_A: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
use super::super::super::interleaved::valued_members;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// The edit roots `first(a)`.
pub struct Root;
impl ApplicationConnectionIdentity for Root {
    const IDENTITY: &'static str = "owned-outputs.late-parent.second-root";
}
pub(super) type Entry = super::super::super::super::program::Entry;
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, hierarchy::Parent> for Root {}
pub(super) type RootConnection = ApplicationConnectionRef<
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
    fn demand_from_source(input: &edit::Intent) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Root> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Root as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}

/// `first(a)` to `guarded-first(a)`, whose incoming prerequisites are model
/// `a`'s members valued 8: `m` selects `first(b)`, any other `first(c)`.
pub struct Guard;
impl ApplicationConnectionIdentity for Guard {
    const IDENTITY: &'static str = "owned-outputs.late-parent.second-guard";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, guarded_first::Feature> for Guard {}
pub(super) type GuardConnection = ApplicationConnectionRef<
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
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.key == "first:a" { vec![guarded_first::Demand("a".into())] } else { Vec::new() })
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
        row: &String,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(if row == "m" { "b" } else { "c" }.into()))
    }
}
/// `first(a)` to `decision-write(b)` (m = b + 1), with no prerequisite.
pub struct WriteEdge;
impl ApplicationConnectionIdentity for WriteEdge {
    const IDENTITY: &'static str = "owned-outputs.late-parent.second-writer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, decision_write::Feature> for WriteEdge {}
pub(super) type WriteEdgeConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    decision_write::Feature,
    decision_write::InputPort,
    WriteEdge,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for WriteEdge {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.key == "first:a" { vec![decision_write::Demand("b".into())] } else { Vec::new() })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for WriteEdge {
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
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
/// `decision-write(b)` to `decision-write-n(k)` (n = k + 1; `k` per court), an occurrence
/// discovered only once `decision-write(b)` has run (no prerequisite: model
/// `b` owns no member).
pub struct SecondEdge;
impl ApplicationConnectionIdentity for SecondEdge {
    const IDENTITY: &'static str = "owned-outputs.late-parent.second-member-writer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_write::Feature, decision_write_n::Feature>
    for SecondEdge
{
}
pub(super) type SecondEdgeConnection = ApplicationConnectionRef<
    Schema,
    decision_write::Feature,
    decision_write::OutputPort,
    decision_write_n::Feature,
    decision_write_n::InputPort,
    SecondEdge,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for SecondEdge {
    type RootDemand = decision_write::Demand;
    type Discovery = query::second_output::Read;
    type Demand = decision_write_n::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::second_output::Read("second:b".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![decision_write_n::Demand(if SECOND_WRITER_A.with(std::cell::Cell::get) { "a" } else { "b" }.into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for SecondEdge {
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
        _: &decision_write_n::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<decision_write::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_write::Demand(row.key.clone()))
    }
}
/// One occurrence component: the guard, the writer and the second writer
/// share one rank, so the second writer runs before the guard's new parent.
pub struct GuardAndWriters;
impl ApplicationOccurrenceComponent<Schema> for GuardAndWriters {
    type Connections = (GuardConnection, WriteEdgeConnection, SecondEdgeConnection);
}
pub type Tree = ApplicationOutputGraph<RootConnection, ApplicationOutputOccurrences<GuardAndWriters>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.second-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.second-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// A world with member `n` (5) beside `m` (5), and model `c` (9). One stage
/// (a = 2) roots `first(a)`, whose occurrence component holds
/// `guarded-first(a)` (refused while `first:b` is absent; its incoming
/// prerequisites are `a`'s members valued 8, `m` selecting `first(b)` and any
/// other `first(c)`) and `decision-write(b)`. The guard has no prerequisite
/// and refuses. `decision-write(b)` sets m = 8: the guard's selection gains row
/// `m`, so it reopens and waits closed on `first(b)`. Only then is
/// `decision-write-n(b)` discovered; it runs before `first(b)` and sets n = 8,
/// adding row `n` while the guard waits. The guard must re-close on the
/// current selection, `{first(b), first(c)}`: `first(c)` derives (90), the guard derives a + 1 = 3, and the edit commits. Red before the
/// fix: the second change was dropped, the guard prepared on `{first(b)}`
/// alone, and `first(c)`, which the final selection requires, was never
/// derived.
#[test]
fn a_second_selection_change_while_a_reopened_task_waits_closed_is_never_dropped() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
        world::Configuration {
            second_member: Some(5),
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("second-selection-change".to_owned())
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
    assert_ne!(courts::native_head(&host), head);
    let value = |key: &str| {
        request
            .query(query::first_output::Read(key.into()))
            .execute()
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value))
    };
    assert_eq!(value("gf:a"), Some(3), "the guard derives once first:b exists");
    assert_eq!(
        value("first:c"),
        Some(90),
        "the guard re-closed on the current selection, which selects first(c)"
    );
}

/// A dropped prerequisite that already derived (PR C C4 r3). The same
/// program over a world where `n` starts at 8. The guard first closes on
/// `first(c)` (row `n`). `decision-write(b)` sets m = 8, so it closes again on
/// `{first(c), first(b)}` before it ever ran, and `first(c)` derives. Then
/// `decision-write-n(a)` sets n = 3 while the guard still waits on `first(b)`,
/// dropping row `n`: a parent it closed on and that already derived. The
/// re-close must remember that selection, so the guard's derivation, which
/// no longer selects `first(c)`, refuses `SupersededSelectionDropped`. Red
/// before the fix: re-closing forgot the derived parent and the edit
/// committed with `first:c` derived for nothing.
#[test]
fn a_closed_task_whose_reselection_drops_a_derived_parent_refuses() {
    SECOND_WRITER_A.with(|flag| flag.set(true));
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
        world::Configuration {
            second_member: Some(8),
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("dropped-derived-parent".to_owned())
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
    super::selection::require_dropped(outcome);
    assert_eq!(courts::native_head(&host), head);
}
