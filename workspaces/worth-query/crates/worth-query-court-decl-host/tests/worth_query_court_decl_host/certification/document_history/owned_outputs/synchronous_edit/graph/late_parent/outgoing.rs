//! A reopened parent whose occurrence-outgoing selection drops a child it
//! derived (PR C r2 blocker 1). Outgoing evidence is replaced at every
//! quiescent refresh, so the drop must refuse in the wave itself.
use super::*;
use family::{decision_input, decision_write};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// `decision-input(a)` links `decision-write(b)` as an occurrence while
/// `first:a` is 7, and nothing at 11. The child names no prerequisite, so it
/// never consumes `first:a`.
pub struct WhileSevenOccurrence;
impl ApplicationConnectionIdentity for WhileSevenOccurrence {
    const IDENTITY: &'static str = "owned-outputs.late-parent.while-seven-occurrence";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, decision_write::Feature>
    for WhileSevenOccurrence
{
}
type WhileSevenOccurrenceConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    WhileSevenOccurrence,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for WhileSevenOccurrence {
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
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.value == 7 {
            vec![decision_write::Demand("b".into())]
        } else {
            Vec::new()
        })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for WhileSevenOccurrence {
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
        // Model `b` has no members: the child names no prerequisite supplier.
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
pub struct WhileSeven;
impl ApplicationOccurrenceComponent<Schema> for WhileSeven {
    type Connections = (WhileSevenOccurrenceConnection,);
}
pub type OccurrenceTree =
    ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<WhileSeven>>;
pub struct OccurrenceDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for OccurrenceDelivery {
    type Mode = ApplicationFixedDeliveryMode<OccurrenceTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.occurrence-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for OccurrenceDelivery {
    type Root = OccurrenceTree;
    type Projection = ApplicationNoProjection;
}
pub struct OccurrenceProgram;
impl ApplicationProgramDefinition<Schema> for OccurrenceProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<OccurrenceTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.occurrence-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<OccurrenceDelivery>();
        specs
    }
}

/// `decision-write(b)` is linked while `first:a` is 7 and writes `m = 9`; the
/// reopened `first:a` is 11 and no longer links it. The edit refuses in the
/// wave, publishing nothing: one commit per stage never derives that child.
#[test]
fn a_reopened_parent_that_drops_an_outgoing_child_refuses() {
    let host = world::install_program_with_other_model::<OccurrenceProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, OccurrenceTree);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-dropped-outgoing".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<OccurrenceTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    super::selection::require_dropped(outcome);
    assert_eq!(courts::native_head(&host), head);
}
