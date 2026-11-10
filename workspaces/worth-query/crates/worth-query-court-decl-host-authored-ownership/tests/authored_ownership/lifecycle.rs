//! Ordinary authored source commands, independent of producer publication machinery.
use super::{co_roles::*, *};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Create {
    pub create_co: bool,
    pub co_anchors: Vec<u64>,
    pub scope: u64,
    pub id: u64,
    pub depth: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Retire {
    pub relation_fault: u8,
    pub corrupt_restoration: bool,
    pub co_anchors: Vec<u64>,
    pub scope: u64,
    pub id: u64,
    pub descendants: Vec<u64>,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Restore {
    pub relation_material: Vec<super::relation_splice::Material>,
    pub retired_entity: Option<super::relation_splice::Coordinate>,
    pub co_values: Vec<u64>,
    pub co_anchors: Vec<u64>,
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
    type Entity = Node<S>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "replacement";
}
pub struct Descendant<S, const I: usize>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Created<S> {
    type Schema = S;
    type Contract = CreatedOutputs;
    type Entity = Node<S>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-created";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for Retired<S> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Node<S>;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-retired";
}
impl<S: Scenario, const I: usize> WorthQueryApplicationOutputRole for Descendant<S, I> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Node<S>;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = ["Rp", "Sc", "Rc", "Sl", "Rl", "C"][I];
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for CreatedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        if <S::Heterogeneous as heterogeneous::Profile>::PRESENT {
            &[

        <Created<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <CoCreated<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <CoCreated<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
            <heterogeneous::BodyCreated<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
]
        } else {
            &[
                <Created<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <CoCreated<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <CoCreated<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
            ]
        };
}
impl<S: Scenario> ApplicationMutationOutputContract<S> for RetiredOutputs {
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = if S::MODE == 72
    {
        &[<coupled::creation_inverse::wire::RetiredMembers<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    } else {
        &[]
    };
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        if <S::Heterogeneous as heterogeneous::Profile>::PRESENT {
            &[

        <Retired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <CoRetired<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <CoRetired<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Extra<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 2> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 3> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 4> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Descendant<S, 5> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
            <heterogeneous::BodyRetired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
]
        } else {
            &[
                <Retired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <CoRetired<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <CoRetired<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Extra<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Descendant<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Descendant<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Descendant<S, 2> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Descendant<S, 3> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Descendant<S, 4> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                <Descendant<S, 5> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
            ]
        };
}
pub struct CreationTargets<S: Scenario> {
    pub scope: Target<S>,
    pub previous: Option<Target<S>>,
}
pub type Target<S> = WorthQueryInvariantMutationTarget<S, Node<S>>;
pub struct RetirementTargets<S: Scenario> {
    pub wire: Vec<coupled::creation_inverse::wire::Member<S>>,
    pub body: Option<WorthQueryInvariantMutationTarget<S, heterogeneous::Body>>,
    pub stage_edges: Vec<(Target<S>, Target<S>)>,
    pub stage_relinks: Vec<(Target<S>, Target<S>)>,
    pub nodes: Vec<Target<S>>,
    pub edges: Vec<(Target<S>, Target<S>)>,
}
worth_query_mutation_binding!(
    pub CreateBinding for Create, schema generic S: Scenario,
    identity "ownership.create.v1", input CreateInput, operation CreateOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision CreationTargets<S>, denial query::UnitBinding, handler identity "ownership.create.handler.v1",
    outputs CreatedOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node<S>,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id<S>,
    field schema::id::<S>(), value_field scope,
);
worth_query_mutation_binding!(
    pub RetireBinding for Retire, schema generic S: Scenario,
    identity "ownership.retire.v1", input RetireInput, operation RetireOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision RetirementTargets<S>, denial query::UnitBinding, handler identity "ownership.retire.handler.v1",
    coupled_outputs |visitor| { if S::COUPLED && S::MODE != 10 { visitor.required::<program::Start>(); }
        if S::TWO_BODIES && S::MODE != 18 { visitor.required::<coupled::multi::SecondStart>(); } },
    outputs RetiredOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node<S>,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id<S>,
    field schema::id::<S>(), value_field scope,
);
worth_query_mutation_binding!(
    pub RestoreBinding for Restore, schema generic S: Scenario,
    identity "ownership.restore.v1", input RestoreInput, operation RestoreOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision super::relation_splice::Restoration<S>, denial query::UnitBinding, handler identity "ownership.restore.handler.v1",
    coupled_outputs |visitor| { if S::COUPLED { visitor.required::<program::Start>(); }
        if S::TWO_BODIES { visitor.required::<coupled::multi::SecondStart>(); } },
    outputs CreatedOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node<S>,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id<S>,
    field schema::id::<S>(), value_field scope,
);
worth_query_mutation_binding!(
    pub PlainBinding for Plain, schema generic S: Scenario,
    identity "ownership.plain.v1", input PlainInput, operation PlainOp,
    result query::UnitBinding, idempotency String, identity "ownership.command.v1",
    decision Vec<Target<S>>, denial query::UnitBinding, handler identity "ownership.plain.handler.v1",
    outputs RetiredOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node<S>,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Id<S>,
    field schema::id::<S>(), value_field scope,
);
impl<const CONTENT: bool> OperationCreates<CreateOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationReads<CreateOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationReads<CreateOp> for NativeNode<CONTENT> {}
impl OperationReads<CreateOp> for Peer {}
impl OperationLinks<CreateOp> for Peer {}
impl<const CONTENT: bool> OperationWrites<CreateOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<CreateOp> for RawValue<CONTENT> {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for CreateOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        let schema = if matches!(S::MODE, 21 | 30 | 39 | 40 | 41 | 60) {
            schema
                .operation_link(Self::reference(), PreviousStage::reference())
                .operation_link(Self::reference(), StageVisible::reference())
        } else {
            schema
        };
        let schema = if S::MODE == 48 {
            schema.operation_link(Self::reference(), Peer::reference())
        } else {
            schema
        };
        schema
            .operation_read_field(Self::reference(), schema::id::<S>())
            .operation_read_entity(Self::reference(), schema::node::<S>())
            .operation_read_relation(Self::reference(), Peer::reference())
            .operation_create(Self::reference(), schema::node::<S>())
            .operation_write(Self::reference(), schema::id::<S>())
            .operation_write(Self::reference(), schema::value::<S>())
    }
}
impl<const CONTENT: bool> OperationCreates<RestoreOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationReads<RestoreOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationReads<RestoreOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<RestoreOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<RestoreOp> for RawValue<CONTENT> {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for RestoreOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        let schema = if matches!(S::MODE, 30 | 39 | 40 | 41) {
            schema
                .operation_read_entity(Self::reference(), schema::node::<S>())
                .operation_read_relation(Self::reference(), PreviousStage::reference())
                .operation_unlink(Self::reference(), PreviousStage::reference())
                .operation_link(Self::reference(), PreviousStage::reference())
        } else {
            schema
        };
        schema
            .operation_read_field(Self::reference(), schema::id::<S>())
            .operation_create(Self::reference(), schema::node::<S>())
            .operation_write(Self::reference(), schema::id::<S>())
            .operation_write(Self::reference(), schema::value::<S>())
    }
}
impl<const CONTENT: bool> OperationReads<RetireOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationReads<RetireOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationReads<RetireOp> for RawValue<CONTENT> {}
impl<const CONTENT: bool> OperationDeletes<RetireOp> for NativeNode<CONTENT> {}
impl OperationReads<RetireOp> for Peer {}
impl OperationUnlinks<RetireOp> for Peer {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for RetireOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        let schema = if matches!(S::MODE, 30 | 39 | 40 | 41) {
            schema.operation_link(Self::reference(), PreviousStage::reference())
        } else {
            schema
        };
        let schema = if matches!(S::MODE, 21 | 30 | 39 | 40 | 41 | 60) {
            let schema = if S::MODE == 39 {
                schema
            } else {
                schema.operation_read_relation(Self::reference(), PreviousStage::reference())
            };
            schema.operation_unlink(Self::reference(), PreviousStage::reference())
        } else {
            schema
        };
        schema
            .operation_read_entity(Self::reference(), schema::node::<S>())
            .operation_read_field(Self::reference(), schema::id::<S>())
            .operation_read_field(Self::reference(), schema::value::<S>())
            .operation_read_relation(Self::reference(), Peer::reference())
            .operation_unlink(Self::reference(), Peer::reference())
            .operation_delete(Self::reference(), schema::node::<S>())
    }
}
impl<const CONTENT: bool> OperationReads<PlainOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationReads<PlainOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationReads<PlainOp> for RawValue<CONTENT> {}
impl<const CONTENT: bool> OperationDeletes<PlainOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationCreates<PlainOp> for NativeNode<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<PlainOp> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<PlainOp> for RawValue<CONTENT> {}
impl<S: Scenario> ApplicationDeclaredOperation<S> for PlainOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .operation_create(Self::reference(), schema::node::<S>())
            .operation_write(Self::reference(), schema::id::<S>())
            .operation_write(Self::reference(), schema::value::<S>())
            .operation_read_entity(Self::reference(), schema::node::<S>())
            .operation_read_field(Self::reference(), schema::id::<S>())
            .operation_read_field(Self::reference(), schema::value::<S>())
            .operation_delete(Self::reference(), schema::node::<S>())
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
