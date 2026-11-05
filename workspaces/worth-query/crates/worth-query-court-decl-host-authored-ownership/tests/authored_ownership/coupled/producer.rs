//! The body's own Preserve drops and replaces its stage-dependent output cell.
use super::super::*;
pub(in super::super) mod handler;
pub(in super::super) mod readiness;
pub use handler::Provider;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub id: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "ownership.body.preserve.input.v1" });
worth_query_operation!(pub Operation for S: Scenario, input InputBinding);
impl OperationReads<Operation> for Node {}
impl OperationReads<Operation> for Id {}
impl OperationReads<Operation> for Value {}
impl OperationReads<Operation> for Peer {}
impl OperationCreates<Operation> for Node {}
impl OperationDeletes<Operation> for Node {}
impl OperationLinks<Operation> for Peer {}
impl OperationUnlinks<Operation> for Peer {}
impl OperationWrites<Operation> for Id {}
impl OperationWrites<Operation> for Value {}
pub struct Outputs;
pub struct Body<S>(PhantomData<fn() -> S>);
pub struct Cell<S>(PhantomData<fn() -> S>);
pub struct RetiredCell<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Body<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for Cell<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "cell-or-child-source";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for RetiredCell<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "previous-cell";
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for Outputs {
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = if S::MODE == 69
    {
        &[<sheet_boundary::SurvivingNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR, <sheet_boundary::RetiredNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    } else {
        &[]
    };
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Body<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Cell<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <RetiredCell<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
type Target<S> = WorthQueryInvariantMutationTarget<S, Node>;
pub struct Decision<S> {
    pub boundary: Option<sheet_boundary::Decision<S>>,
    pub next_cell: u64,
    pub cell_value: u64,
    pub source: Target<S>,
    pub body: Target<S>,
    pub previous_cell: Target<S>,
    pub previous_edges: Vec<(Target<S>, Target<S>)>,
    pub own_edges: Vec<(Target<S>, Target<S>)>,
    pub stage: Option<Target<S>>,
    pub other_stage: Option<Target<S>>,
    pub visibility_target: Option<Target<S>>,
    pub foreign_member: Option<Target<S>>,
}
worth_query_mutation_binding!(
    pub Binding for Input, schema generic S: Scenario,
    identity "ownership.body.preserve.v1", input InputBinding, operation Operation,
    result query::UnitBinding, idempotency u64, identity "ownership.body.key.v1",
    decision Decision<S>, denial query::UnitBinding, handler identity "ownership.body.preserve.handler.v1",
    outputs Outputs, source ApplicationQueryMutationSource<query::Query>,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Node, Facts, Id, u64, ReadOnly, NoApplicationUnit,
    field Id::reference(), value_field id,
);
pub struct Producer<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationProducerBinding<S> for Producer<S> {
    type Operation = Binding<S>;
    type OutputFamily = super::super::producer::Family;
    type Provider = Provider;
    type OutputRole = Body<S>;
    const IDENTITY: &'static str = "ownership.body.preserve.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
        &[WorthQueryProducerApplicability::new(
            "node",
            WorthQueryProducerLifecyclePosture::Preserve,
        )];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "actual-stage-set";
    const OWNERSHIP: ProducerPublicationOwnership =
        ProducerPublicationOwnership::SourceScopeCascade;
}
impl<S: Scenario> WorthQueryApplicationProducerProvider<S, Producer<S>> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "ownership.body.preserve.provider.v1";
    fn operation_input(&self, source: &query::Row) -> Input {
        Input { id: source.id }
    }
    fn idempotency_key(&self, _: &query::Row, source: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source[..8].try_into().unwrap())
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = Operation::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_read_entity(op, Node::reference())
        .operation_read_field(op, Id::reference())
        .operation_read_field(op, Value::reference())
        .operation_read_relation(op, Peer::reference())
        .operation_create(op, Node::reference())
        .operation_delete(op, Node::reference())
        .operation_link(op, Peer::reference())
        .operation_unlink(op, Peer::reference())
        .operation_write(op, Id::reference())
        .operation_write(op, Value::reference())
        .application_mutation_binding::<Binding<S>>()
}
