//! R3 settlement of a consumer that reads its parent through the public
//! `current_output` (PR C C3b r1): its consumed-output evidence settles to
//! the replacement and publishes against it, or it reopens.
use super::*;
use family::saturated_consumer;

/// `saturated-input(a)` to `saturated-consumer(c)`, which reads `saturated:a`
/// through `current_output`, with the saturated parent as its strict
/// prerequisite supplier.
pub struct Reader;
impl ApplicationConnectionIdentity for Reader {
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-current-output";
}
impl
    ApplicationOccurrenceConnectionBinding<
        Schema,
        saturated_input::Feature,
        saturated_consumer::Feature,
    > for Reader
{
}
type ReaderConnection = ApplicationConnectionRef<
    Schema,
    saturated_input::Feature,
    saturated_input::OutputPort,
    saturated_consumer::Feature,
    saturated_consumer::InputPort,
    Reader,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Reader {
    type RootDemand = saturated_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = saturated_consumer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("saturated:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![saturated_consumer::Demand("c".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Reader {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &saturated_consumer::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("saturated:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<saturated_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(saturated_input::Demand(row.key.clone()))
    }
}
pub struct ReadThenWrite;
impl ApplicationOccurrenceComponent<Schema> for ReadThenWrite {
    type Connections = (ReaderConnection, WriterConnection);
}
pub type ReaderTree =
    ApplicationOutputGraph<StartConnection, ApplicationOutputOccurrences<ReadThenWrite>>;
pub struct ReaderDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for ReaderDelivery {
    type Mode = ApplicationFixedDeliveryMode<ReaderTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.settlement-current-output-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for ReaderDelivery {
    type Root = ReaderTree;
    type Projection = ApplicationNoProjection;
}
pub struct ReaderProgram;
impl ApplicationProgramDefinition<Schema> for ReaderProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ReaderTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "owned-outputs.late-parent.settlement-current-output-program",
    );
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::super::program::entry_spec::<ReaderDelivery>();
        specs
    }
}

/// The default world (a = 2, b = 7, m = 5) plus model `c` (3).
fn install() -> WorthQueryProgramApplicationRuntime<Schema, ReaderProgram> {
    let declaration = Schema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, ReaderProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| {
            world::seed(graph, installed, 2, Some(7))?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Model::reference(),
                    WorthQueryApplicationEntityKey::new("c").unwrap(),
                )
                .field(ModelKey::at("ModelKey"), "c".to_owned())
                .field(ModelValue::at("ModelValue"), 3u64),
            )
        },
    )
    .unwrap()
}
fn edit_stages(
    host: &WorthQueryProgramApplicationRuntime<Schema, ReaderProgram>,
    request: &family::Request<'_, '_, '_>,
    idempotency: &str,
    stages: &[(&str, u64)],
) -> Outcome {
    let mut edit = request
        .edit_envelope()
        .idempotency(idempotency.to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    for (key, value) in stages {
        edit.push_stage::<ReaderTree, _>(edit::Intent {
            key: (*key).into(),
            value: *value,
        })
        .unwrap();
    }
    edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

/// One stage (`b = 8`) roots `saturated-input(a)` (2 + min(5, 5) = 7). Its
/// consumer `saturated-consumer(c)` reads it through `current_output`
/// (Appended evidence) and derives sc:c = 7; `decision-write(b)` writes
/// m = 9. The parent reopens and derives 2 + min(9, 5) = 7 again; the
/// consumer settles to that replacement, which is appended after it. The
/// edit commits (the replacement's lineage slot records before the
/// consumer's publication settles against it) with saturated:a = 7 and
/// sc:c = 7, the parent twice, the consumer and the writer once each.
#[test]
fn a_current_output_consumer_settles_to_a_replacement_appended_after_it() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = edit_stages(&host, &request, "settlement-current-output", &[("b", 8)]);
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert_eq!(first_output(&request, "saturated:a"), 7);
    assert_eq!(first_output(&request, "sc:c"), 7);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        4,
        "{events:#?}"
    );
    assert!(
        events.iter().any(|event| event.starts_with("settle consumer")),
        "{events:#?}"
    );
}

/// `b = 4` commits saturated:a = 7 (Initial) and sc:c = 7; `a = 3` commits
/// saturated:a = 3 + 5 = 8 (Preserve) and sc:c = 8. The edit `c = 4, b = 8`
/// adopts the parent verified; the consumer runs (its source changed) and
/// reads it through `current_output` as a committed output (Recorded
/// evidence); `decision-write(b)` writes m = 9 and the parent's Preserve
/// round derives 3 + min(9, 5) = 8 again. Recorded evidence has no
/// settlement transition, so the consumer reopens and reads the
/// replacement; the edit commits saturated:a = 8, sc:c = 8. A later edit
/// that changes nothing (`c = 4` again) runs no producer: the consumer's
/// committed lineage names the replacement, not the retired parent output.
#[test]
fn a_recorded_current_output_consumer_reopens_and_publishes_against_the_replacement() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for (idempotency, key, value) in [("current-born", "b", 4), ("current-preserved", "a", 3)] {
        let committed = edit_stages(&host, &request, idempotency, &[(key, value)]);
        assert!(matches!(committed, Outcome::Committed(_)), "{committed:?}");
    }
    assert_eq!(first_output(&request, "saturated:a"), 8);
    assert_eq!(first_output(&request, "sc:c"), 8);
    let outcome = edit_stages(&host, &request, "current-recorded", &[("c", 4), ("b", 8)]);
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    assert_eq!(first_output(&request, "saturated:a"), 8);
    assert_eq!(first_output(&request, "sc:c"), 8);
    assert!(
        !events.iter().any(|event| event.starts_with("settle consumer")),
        "Recorded evidence never settles: {events:#?}"
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let healthy = edit_stages(&host, &request, "current-healthy", &[("c", 4)]);
    assert!(matches!(healthy, Outcome::Committed(_)), "{healthy:?}");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        0,
        "every output is current after the reopened consumer's publication"
    );
}

