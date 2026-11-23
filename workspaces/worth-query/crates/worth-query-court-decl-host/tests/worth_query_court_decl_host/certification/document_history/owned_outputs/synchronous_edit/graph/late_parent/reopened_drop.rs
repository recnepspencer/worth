//! A pure prerequisite parent its only selector drops after it derived (PR
//! C C4 r5, blocker 1). The reachable public order, pinned here: the member
//! writer, discovered beside the selector, runs before the parent, so the
//! parent derives over the written member and the later drop finds it done.
//! The reopened-at-drop variant needs a parent whose own decision read
//! changes between its derivation and the drop with its selector still
//! undone; no public fixture orders that (C4 report, r5). The derived-parent
//! predicate (`has_derived`) and the orphan backstop both guard it.
use super::reopen_closed::{SecondEdgeConnection, SECOND_WRITER_A};
use super::*;
use super::super::super::interleaved::valued_members;
use family::{decision_input, decision_write, ordinary_input};

/// `decision-input(a)` to `ordinary-input(b)`, whose incoming prerequisites
/// are model `a`'s members valued 8: `n` selects `decision-input(c)` (which
/// reads member `m`), any other `decision-input(d)`.
pub struct Pick;
impl ApplicationConnectionIdentity for Pick {
    const IDENTITY: &'static str = "owned-outputs.late-parent.reopened-pick";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, ordinary_input::Feature> for Pick {}
type PickConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    ordinary_input::Feature,
    ordinary_input::InputPort,
    Pick,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Pick {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = ordinary_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.key == "first:a" { vec![ordinary_input::Demand("b".into())] } else { Vec::new() })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Pick {
    type Prerequisites = valued_members::Read;
    type PrerequisiteRow = valued_members::ChildBinding;
    type Selection = valued_members::ValuedMembers;
    fn prerequisite_rows() -> Self::Selection {
        valued_members::valued()
    }
    fn prerequisites_from_target(
        _: &ordinary_input::Demand,
    ) -> Result<valued_members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(valued_members::Read("a".into()))
    }
    fn parent_from_prerequisite(
        row: &String,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(if row == "n" { "c" } else { "d" }.into()))
    }
}
/// `decision-input(a)` to `decision-write(b)` (m = b + 1), with no
/// prerequisite (model `b` owns no member).
pub struct Writer;
impl ApplicationConnectionIdentity for Writer {
    const IDENTITY: &'static str = "owned-outputs.late-parent.reopened-writer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, decision_write::Feature> for Writer {}
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
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Writer {
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
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
/// The writer edge, the pick and the second writer in one component.
pub struct Component;
impl ApplicationOccurrenceComponent<Schema> for Component {
    type Connections = (WriterConnection, PickConnection, SecondEdgeConnection);
}
pub type Tree = ApplicationOutputGraph<super::program::Connection, ApplicationOutputOccurrences<Component>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.reopened-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.reopened-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (b = 8) roots `decision-input(a)` (2 + m). Its children are
/// `decision-write(b)` (m = 9), `ordinary-input(b)`, which selects
/// `decision-input(c)` (9 + m) through row `n` = 8, and, once the writer
/// runs, `decision-write-n(a)` (n = 3). The writer runs first; the parent
/// derives over m = 9; the second writer's n = 3 drops it while its selector
/// still waits. A selection dropped after it derived refuses
/// `SupersededSelectionDropped`, and nothing commits.
#[test]
fn a_pure_parent_its_selector_drops_after_it_derived_refuses() {
    SECOND_WRITER_A.with(|flag| flag.set(true));
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
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
        .idempotency("reopened-drop".to_owned())
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
    let detail = format!("{outcome:?}");
    assert!(detail.contains("SupersededSelectionDropped"), "{detail}; {events:#?}");
    assert_eq!(courts::native_head(&host), head);
}
