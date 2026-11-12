//! Same family/source with a different canonical partition is never a prepared hit.
use super::*;
#[derive(Clone)]
pub struct OtherParameters(String);
impl WorthQueryApplicationOutputDemand<Schema> for OtherParameters {
    type OutputFamily = first::Family;
    fn source_intent(&self) -> query::producer_model::Read {
        query::producer_model::Read {
            key: self.0.clone(),
            expected: "absent-partition".into(),
        }
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.graph.other-partition";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Discover
{
}
type Connection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = OtherParameters;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<OtherParameters>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![OtherParameters(
            row.key.strip_prefix("first:").unwrap().into(),
        )])
    }
}
type Tree = ApplicationOutputGraph<
    program::Connection,
    ApplicationOutputEdge<Connection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.graph.other-partition-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.graph.other-partition-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"other-canonical-partition".to_owned())
        .execute_performed::<Program, Tree>(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("the foreign partition must not reuse the parent's writer")
    };
    let source = cause
        .coupled_producer_denial()
        .expect("actual source query refusal");
    assert_eq!(
        source.kind(),
        WorthQueryOutputDemandDenialKind::ProducerUnavailable
    );
    assert!(
        source.subject().contains("owned-outputs.first.initial"),
        "{source:?}"
    );
    assert!(
        source.subject().contains("CardinalityMismatch"),
        "{source:?}"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "only the actual parent ran"
    );
    super::super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .is_err());
}
#[test]
fn a_different_canonical_partition_cannot_select_a_prepared_writer() {
    exercise(false)
}
#[test]
fn worker_only_a_different_canonical_partition_cannot_select_a_prepared_writer() {
    exercise(true)
}
