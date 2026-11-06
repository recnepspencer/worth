//! Ordinary authored source commands, independent of producer publication machinery.
use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Create {
    pub scope: u64,
    pub id: u64,
    pub depth: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Retire {
    pub scope: u64,
    pub id: u64,
    pub descendants: Vec<u64>,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Restore {
    pub scope: u64,
    pub id: u64,
    pub value: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Plain {
    pub replacement: bool,
    pub scope: u64,
    pub id: u64,
    pub descendants: Vec<u64>,
}
worth_query_structured_value_binding!(pub CreateInput for Create { identity: "ownership.authored.create.input.v1" });
worth_query_structured_value_binding!(pub RetireInput for Retire { identity: "ownership.authored.retire.input.v1" });
worth_query_structured_value_binding!(pub RestoreInput for Restore { identity: "ownership.authored.restore.input.v1" });
worth_query_structured_value_binding!(pub PlainInput for Plain { identity: "ownership.plain.retire.input.v1" });
worth_query_operation!(pub CreateOp for S: Scenario, input CreateInput);
worth_query_operation!(pub RetireOp for S: Scenario, input RetireInput);
worth_query_operation!(pub RestoreOp for S: Scenario, input RestoreInput);
worth_query_operation!(pub PlainOp for S: Scenario, input PlainInput);
pub struct CreatedOutputs;
pub struct RetiredOutputs;
pub struct Created<S>(PhantomData<fn() -> S>);
pub struct Retired<S>(PhantomData<fn() -> S>);
pub struct Extra<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Extra<S> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Node;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "replacement";
}
pub struct Descendant<S, const I: usize>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Created<S> {
    type Schema = S;
    type Contract = CreatedOutputs;
    type Entity = Node;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-created";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for Retired<S> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Node;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-retired";
}
impl<S: Scenario, const I: usize> WorthQueryApplicationOutputRole for Descendant<S, I> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Node;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = ["Rp", "Sc", "Rc", "Sl", "Rl", "C"][I];
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for CreatedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<Created<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for RetiredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Retired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Extra<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 2> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 3> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 4> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 5> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
pub type Target<S> = WorthQueryInvariantMutationTarget<S, Node>;
worth_query_mutation_binding!(
    pub CreateBinding for Create, schema generic S: Scenario,
    identity "ownership.create.v1", input CreateInput, operation CreateOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision Target<S>, denial query::UnitBinding, handler identity "ownership.create.handler.v1",
    outputs CreatedOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id,
    field Id::at("Id"), value_field scope,
);
worth_query_mutation_binding!(
    pub RetireBinding for Retire, schema generic S: Scenario,
    identity "ownership.retire.v1", input RetireInput, operation RetireOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision Vec<Target<S>>, denial query::UnitBinding, handler identity "ownership.retire.handler.v1",
    outputs RetiredOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id,
    field Id::at("Id"), value_field scope,
);
worth_query_mutation_binding!(
    pub RestoreBinding for Restore, schema generic S: Scenario,
    identity "ownership.restore.v1", input RestoreInput, operation RestoreOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision Target<S>, denial query::UnitBinding, handler identity "ownership.restore.handler.v1",
    outputs CreatedOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id,
    field Id::at("Id"), value_field scope,
);
worth_query_mutation_binding!(
    pub PlainBinding for Plain, schema generic S: Scenario,
    identity "ownership.plain.v1", input PlainInput, operation PlainOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision Vec<Target<S>>, denial query::UnitBinding, handler identity "ownership.plain.handler.v1",
    outputs RetiredOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id,
    field Id::at("Id"), value_field scope,
);
impl OperationCreates<CreateOp> for Node {}
impl OperationReads<CreateOp> for Id {}
impl OperationWrites<CreateOp> for Id {}
impl OperationWrites<CreateOp> for Value {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for CreateOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .operation_read_field(Self::reference(), Id::at("Id"))
            .operation_create(Self::reference(), Node::reference())
            .operation_write(Self::reference(), Id::at("Id"))
            .operation_write(Self::reference(), Value::at("Value"))
    }
}
impl OperationCreates<RestoreOp> for Node {}
impl OperationReads<RestoreOp> for Id {}
impl OperationWrites<RestoreOp> for Id {}
impl OperationWrites<RestoreOp> for Value {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for RestoreOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .operation_read_field(Self::reference(), Id::at("Id"))
            .operation_create(Self::reference(), Node::reference())
            .operation_write(Self::reference(), Id::at("Id"))
            .operation_write(Self::reference(), Value::at("Value"))
    }
}
impl OperationReads<RetireOp> for Node {}
impl OperationReads<RetireOp> for Id {}
impl OperationReads<RetireOp> for Value {}
impl OperationDeletes<RetireOp> for Node {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for RetireOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .operation_read_entity(Self::reference(), Node::reference())
            .operation_read_field(Self::reference(), Id::at("Id"))
            .operation_read_field(Self::reference(), Value::at("Value"))
            .operation_delete(Self::reference(), Node::reference())
    }
}
impl OperationReads<PlainOp> for Node {}
impl OperationReads<PlainOp> for Id {}
impl OperationReads<PlainOp> for Value {}
impl OperationDeletes<PlainOp> for Node {}
impl OperationCreates<PlainOp> for Node {}
impl OperationWrites<PlainOp> for Id {}
impl OperationWrites<PlainOp> for Value {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for PlainOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .operation_create(Self::reference(), Node::reference())
            .operation_write(Self::reference(), Id::at("Id"))
            .operation_write(Self::reference(), Value::at("Value"))
            .operation_read_entity(Self::reference(), Node::reference())
            .operation_read_field(Self::reference(), Id::at("Id"))
            .operation_read_field(Self::reference(), Value::at("Value"))
            .operation_delete(Self::reference(), Node::reference())
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .declared_operation(
            RestoreOp::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<RestoreBinding<S>>()
        .declared_operation(
            PlainOp::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<PlainBinding<S>>()
}
