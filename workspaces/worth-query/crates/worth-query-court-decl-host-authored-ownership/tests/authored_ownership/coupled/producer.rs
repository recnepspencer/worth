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
impl<const CONTENT: bool> OperationReads<Operation> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationReads<Operation> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationReads<Operation> for RawValue<CONTENT> {}
impl OperationReads<Operation> for Peer {}
impl<const CONTENT: bool> OperationCreates<Operation> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationDeletes<Operation> for NativeNode<CONTENT> {}
impl OperationLinks<Operation> for Peer {}
impl OperationUnlinks<Operation> for Peer {}
impl<const CONTENT: bool> OperationWrites<Operation> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<Operation> for RawValue<CONTENT> {}
pub struct Outputs;
pub struct Body<S>(PhantomData<fn() -> S>);
pub struct Cell<S>(PhantomData<fn() -> S>);
pub struct RetiredCell<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Body<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node<S>;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for Cell<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node<S>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "cell-or-child-source";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for RetiredCell<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node<S>;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "previous-cell";
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for Outputs {
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = if matches!(
        S::MODE,
        69 | 72
    ) {
        &[<sheet_boundary::SurvivingNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR, <sheet_boundary::RetiredNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    } else if S::MODE == 70 {
        &[
            <sheet_boundary::SurvivingNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
            <sheet_boundary::RetiredNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
            <sheet_boundary::split_walk::SurvivingFork<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
            <sheet_boundary::split_walk::RetiredFork<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
        ]
    } else {
        &[]
    };
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Body<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Cell<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <RetiredCell<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
type Target<S> = WorthQueryInvariantMutationTarget<S, Node<S>>;
pub struct Decision<S: Scenario> {
    pub boundary: Option<<S::Boundary as sheet_boundary::Profile>::Decision<S>>,
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
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node<S>,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id<S>, field schema::id::<S>(), value_field id,
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
        .operation_read_entity(op, schema::node::<S>())
        .operation_read_field(op, schema::id::<S>())
        .operation_read_field(op, schema::value::<S>())
        .operation_read_relation(op, Peer::reference())
        .operation_create(op, schema::node::<S>())
        .operation_delete(op, schema::node::<S>())
        .operation_link(op, Peer::reference())
        .operation_unlink(op, Peer::reference())
        .operation_write(op, schema::id::<S>())
        .operation_write(op, schema::value::<S>())
        .application_mutation_binding::<Binding<S>>()
}
