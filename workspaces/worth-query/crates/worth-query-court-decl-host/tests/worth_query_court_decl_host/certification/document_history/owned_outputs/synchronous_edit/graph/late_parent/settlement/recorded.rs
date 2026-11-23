//! R3 settlement of a recorded parent's consuming writer (#239), and the
//! equal-payload, different-custody reopen.
use super::*;

/// `saturated-input(a)` to `decision-write(b)` with the saturated parent as
/// the writer's strict prerequisite supplier: the writer consumes the parent
/// whose decision read it changes (#239 when the parent is recorded).
pub struct ConsumingWriter;
impl ApplicationConnectionIdentity for ConsumingWriter {
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-consuming-writer";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, saturated_input::Feature, decision_write::Feature>
    for ConsumingWriter
{
}
type ConsumingWriterConnection = ApplicationConnectionRef<
    Schema,
    saturated_input::Feature,
    saturated_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    ConsumingWriter,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for ConsumingWriter {
    type RootDemand = saturated_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("saturated:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![decision_write::Demand("b".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for ConsumingWriter {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &decision_write::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("saturated:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<saturated_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(saturated_input::Demand(row.key.clone()))
    }
}
pub struct Recorded;
impl ApplicationOccurrenceComponent<Schema> for Recorded {
    type Connections = (ConsumingWriterConnection,);
}
pub type RecordedTree =
    ApplicationOutputGraph<StartConnection, ApplicationOutputOccurrences<Recorded>>;
pub struct RecordedDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for RecordedDelivery {
    type Mode = ApplicationFixedDeliveryMode<RecordedTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-recorded-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for RecordedDelivery {
    type Root = RecordedTree;
    type Projection = ApplicationNoProjection;
}
pub struct RecordedProgram;
impl ApplicationProgramDefinition<Schema> for RecordedProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<RecordedTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.settlement-recorded-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<RecordedDelivery>();
        specs
    }
}

fn stage(
    host: &WorthQueryProgramApplicationRuntime<Schema, RecordedProgram>,
    request: &family::Request<'_, '_, '_>,
    idempotency: &str,
    key: &str,
    value: u64,
) -> Outcome {
    let mut edit = request
        .edit_envelope()
        .idempotency(idempotency.to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<RecordedTree, _>(edit::Intent {
        key: key.into(),
        value,
    })
    .unwrap();
    edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
fn recorded_host() -> WorthQueryProgramApplicationRuntime<Schema, RecordedProgram> {
    world::install_program_with_other_model::<RecordedProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    )
}
fn second_b(request: &family::Request<'_, '_, '_>) -> u64 {
    request
        .query(query::second_output::Read("second:b".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

/// `b = 4` commits `saturated:a` through its Initial producer (2 + min(5, 5) =
/// 7) and its consuming writer (m = 4 + 1 = 5); then `a = 3` derives it again
/// through its Preserve producer (3 + 5 = 8). The edit `b = 8` adopts the
/// parent verified; its consuming writer derives (b changed) and writes
/// m = 9, which the recorded parent read. Qualified use reopens the parent,
/// whose Preserve round derives again: 3 + min(9, 5) = 8, exactly what the
/// writer consumed, under the same producer. The writer settles and does not
/// run again: the writer and the reopened parent run once each (2 producer
/// contacts), and the edit commits the sequential world (saturated:a = 8,
/// second:b = 1008).
#[test]
fn an_unchanged_replacement_of_a_recorded_parent_settles_its_consuming_writer() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = recorded_host();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for (idempotency, key, value) in [("recorded-born", "b", 4), ("recorded-preserved", "a", 3)] {
        let committed = stage(&host, &request, idempotency, key, value);
        assert!(matches!(committed, Outcome::Committed(_)), "{committed:?}");
    }
    assert_eq!(first_output(&request, "saturated:a"), 8);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = stage(&host, &request, "late-parent-recorded", "b", 8);
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert_eq!(first_output(&request, "saturated:a"), 8);
    assert_eq!(second_b(&request), 1008);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        2,
        "the writer and the reopened parent; the writer settles: {events:#?}"
    );
    assert!(
        events.iter().any(|event| event.starts_with("settle consumer")),
        "{events:#?}"
    );
}

/// The same edit over a parent recorded by its Initial producer: `b = 4`
/// commits `saturated:a` = 7. The reopened parent's replacement (Preserve,
/// 2 + min(9, 5) = 7) has an equal payload under a different custody, its
/// producer. Plan §1.2: "Equal value alone is never enough. Custody, role,
/// generation, readiness and demand key are part of the comparison." So R3
/// reopens the consuming writer instead of settling it. The writer runs
/// again on the replacement, writes m = 9 again, and the edit commits the
/// sequential world (saturated:a = 7, second:b = 1008) with 3 producer
/// contacts.
#[test]
fn an_equal_payload_under_another_custody_reopens_its_consuming_writer() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = recorded_host();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let committed = stage(&host, &request, "custody-born", "b", 4);
    assert!(matches!(committed, Outcome::Committed(_)), "{committed:?}");
    assert_eq!(first_output(&request, "saturated:a"), 7);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = stage(&host, &request, "late-parent-custody", "b", 8);
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert_eq!(first_output(&request, "saturated:a"), 7);
    assert_eq!(second_b(&request), 1008);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        3,
        "the writer, the reopened parent, and the writer again: {events:#?}"
    );
    assert!(
        events.iter().any(|event| event.starts_with("reopen consumer")),
        "{events:#?}"
    );
    assert!(
        !events.iter().any(|event| event.starts_with("settle consumer")),
        "{events:#?}"
    );
}
