//! One append that changes both a parent's selection and its own decision
//! read (PR C C4 r1 blocker 1): the selection's re-discovery and the parent's
//! reopen must verify, together, every child it selected before. A second
//! reopen never forgets what the first one recorded.
use super::*;
use family::{decision_input, decision_write};

/// `decision-write(b)` while member `m` is 5, read straight from `m`.
pub struct WhileFive;
impl ApplicationConnectionIdentity for WhileFive {
    const IDENTITY: &'static str = "owned-outputs.late-parent.while-five";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, decision_write::Feature>
    for WhileFive
{
}
pub type WhileFiveConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    WhileFive,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for WhileFive {
    type RootDemand = decision_input::Demand;
    type Discovery = query::member_public::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::member_public::Read("m".into()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.value == 5 {
            vec![decision_write::Demand("b".into())]
        } else {
            Vec::new()
        })
    }
}
pub type MergeTree = ApplicationOutputGraph<
    program::Connection,
    ApplicationOutputEdge<WhileFiveConnection, ApplicationOutputLeaf>,
>;
pub struct MergeDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for MergeDelivery {
    type Mode = ApplicationFixedDeliveryMode<MergeTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.merge-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for MergeDelivery {
    type Root = MergeTree;
    type Projection = ApplicationNoProjection;
}
pub struct MergeProgram;
impl ApplicationProgramDefinition<Schema> for MergeProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<MergeTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.merge-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<MergeDelivery>();
        specs
    }
}

/// One stage roots `first(a)` (a + m = 2 + 5 = 7), which selects
/// `decision-write(b)` because `m` is 5. That writer sets `m` = b + 1 = 8: its
/// single append changes the parent's selection (it now selects nothing) and
/// its decision read. The selection selects again first and records the
/// writer as selected before; the parent's reopen must keep that record, so
/// its re-derivation (10) drops the writer and the edit refuses
/// `SupersededSelectionDropped`. One commit per stage refuses too: the
/// parent's own child changed what it read. Red before the fix: the reopen
/// overwrote the record with an empty one and the edit committed the dropped
/// writer.
#[test]
fn one_append_changing_a_parents_selection_and_its_read_still_refuses_the_dropped_child() {
    let host = world::install_program_with_other_model::<MergeProgram>(
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
        .idempotency("late-parent-merged-reopen".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<MergeTree, _>(edit::Intent {
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
