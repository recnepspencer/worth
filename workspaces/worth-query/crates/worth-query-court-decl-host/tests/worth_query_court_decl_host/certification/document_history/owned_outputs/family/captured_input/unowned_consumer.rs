//! An unowned consumer that changes its owned supplier's decision read is
//! not reopenable as that supplier's consumer: it cannot derive its route
//! again. The supplier's changed read stays with final qualification (PR C
//! C3a r4b note 1).
use super::super::super::synchronous_edit::{envelope_fixture as fixture, interleaved};
use super::program::{AnchorConnection, Root, SeedConnection};
use super::*;
use interleaved::component_carriage::parent as owner;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};

/// `captured-input(capture:child)` to `ordinary-input(carriage:d)`, with the
/// captured reader as the consumer's strict prerequisite supplier. The
/// consumer is an ordinary participant: it creates `first:carriage:d` and
/// owns no route.
pub struct Consumer;
impl ApplicationConnectionIdentity for Consumer {
    const IDENTITY: &'static str = "captured-input.unowned-consumer";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::Feature, ordinary_input::Feature>
    for Consumer
{
}
type ConsumerConnection = ApplicationConnectionRef<
    Schema,
    program::Feature,
    program::Output,
    ordinary_input::Feature,
    ordinary_input::InputPort,
    Consumer,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Consumer {
    type RootDemand = Demand;
    type Discovery = query::first_output::Read;
    type Demand = ordinary_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("captured:capture:child".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<ordinary_input::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![ordinary_input::Demand("carriage:d".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Consumer {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &ordinary_input::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("captured:capture:child".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(row.key.clone()))
    }
}
pub struct Component;
impl ApplicationOccurrenceComponent<Schema> for Component {
    type Connections = (
        ApplicationCapturedInputConnection<SeedConnection, Capture>,
        ApplicationCapturedInputConnection<AnchorConnection, Capture>,
        ConsumerConnection,
    );
}
pub type Tree = ApplicationOutputGraph<Root, ApplicationOutputOccurrences<Component>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.unowned-consumer");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<Tree>>()
    }
}

/// One stage roots `captured-input(capture:child)` (a = 1, carriage:late = 5),
/// whose undeclared anchor looks up `first:carriage:d` and finds none. It discovers its consumer
/// `ordinary-input(carriage:d)`, which creates `first:carriage:d`: the
/// consumer's own append changes its supplier's decision read. The consumer
/// is ordinary and cannot derive again, so it is not reopenable as the
/// supplier's consumer; the supplier is not reopened, and final
/// qualification refuses `CoupledReadSupportChanged`. Reopening it would
/// reopen the consumer with it (R3) and meet Native's owned-route refusal.
#[test]
fn an_unowned_consumer_changing_its_suppliers_decision_read_refuses_at_final_qualification() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let declaration = Schema::declaration().unwrap();
    // Only the captured reader's own records: no `carriage:b`, so nothing
    // derives `first(carriage:d)` and the consumer creates it.
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (world::Configuration {
            envelope_records: true,
            captured_inputs: true,
            captured_undeclared_anchor: true,
            ..Default::default()
        },),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| {
            world::seed(graph, installed, 1, None)?;
            fixture::seed(graph)?;
            for (key, value) in [("carriage:late", 5u64), ("carriage:d", 4)] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Model::reference(),
                        WorthQueryApplicationEntityKey::new(key).unwrap(),
                    )
                    .field(ModelKey::at("ModelKey"), key.to_owned())
                    .field(ModelValue::at("ModelValue"), value),
                )?;
            }
            program::seed(graph, None, true)
        },
    )
    .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope())
        .idempotency("captured-unowned-consumer".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(fixture::model::Intent {
        key: "capture:child".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("the consumer's change to its supplier's read must refuse: {outcome:?}")
    };
    let failure = denial
        .invariant_execution_failure()
        .unwrap_or_else(|| panic!("final candidate support refuses: {denial:?}"));
    assert_eq!(
        failure.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged,
        "{failure:?}"
    );
    assert!(
        failure.detail().contains("indexed-entity-selection"),
        "{failure:?}"
    );
    assert!(
        failure
            .detail()
            .contains("later writer producer owned-outputs.ordinary-input.initial"),
        "{failure:?}"
    );
}
