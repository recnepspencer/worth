//! Spawn bears a node and its link; Despawn retires exactly what it names.
use super::*;
/// `inverse_fault` selects a deliberately wrong inverse: 1 also removes the
/// next spawn's link, 2 reads an undeclared relation pre-image.
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Spawn {
    pub id: u64,
    pub part: bool,
    pub link: bool,
    pub inverse_fault: u8,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Despawn {
    pub id: u64,
    pub part: Option<u64>,
    pub foreign: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct Done;
#[derive(Clone, Debug)]
pub struct Refused;
/// Every node is created and retired in the context of the seeded scope.
pub const SCOPE: u64 = 1;
worth_query_structured_value_binding!(pub SpawnInput for Spawn { identity: "relation-only-inverse.spawn.input.v1" });
worth_query_structured_value_binding!(pub DespawnInput for Despawn { identity: "relation-only-inverse.despawn.input.v1" });
worth_query_structured_value_binding!(pub DoneBinding for Done { identity: "relation-only-inverse.done.v1" });
worth_query_structured_value_binding!(pub RefusedBinding for Refused { identity: "relation-only-inverse.refused.v1" });
worth_query_operation!(pub SpawnOp for Schema, input SpawnInput);
worth_query_operation!(pub DespawnOp for Schema, input DespawnInput);
macro_rules! output_role {
    ($role:ident, $contract:ident, $action:ident, $cardinality:ident, $name:literal) => {
        pub struct $role;
        impl WorthQueryApplicationOutputRole for $role {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Node;
            type Action = $action;
            type Cardinality = $cardinality;
            const NAME: &'static str = $name;
        }
    };
}
pub struct SpawnOutputs;
pub struct DespawnOutputs;
output_role!(
    Spawned,
    SpawnOutputs,
    WorthQueryCreateOutput,
    WorthQueryExactlyOneOutput,
    "spawned"
);
output_role!(
    SpawnedPart,
    SpawnOutputs,
    WorthQueryCreateOutput,
    WorthQueryAtMostOneOutput,
    "spawned-part"
);
output_role!(
    Despawned,
    DespawnOutputs,
    WorthQueryRetireOutput,
    WorthQueryExactlyOneOutput,
    "despawned"
);
output_role!(
    DespawnedPart,
    DespawnOutputs,
    WorthQueryRetireOutput,
    WorthQueryAtMostOneOutput,
    "despawned-part"
);
impl ApplicationMutationOutputContract<Schema> for SpawnOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Spawned as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <SpawnedPart as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
impl ApplicationMutationOutputContract<Schema> for DespawnOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Despawned as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <DespawnedPart as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
pub type Target = WorthQueryInvariantMutationTarget<Schema, Node>;
pub struct DespawnTargets {
    pub spawned: Target,
    pub part: Option<Target>,
    pub links: Vec<(Target, Target)>,
}
fn spawn_scope(_: &Spawn) -> u64 {
    SCOPE
}
fn despawn_scope(_: &Despawn) -> u64 {
    SCOPE
}
worth_query_mutation_binding!(pub SpawnBinding for Spawn, schema Schema,
    identity "relation-only-inverse.spawn.v1", input SpawnInput,
    operation SpawnOp, result DoneBinding,
    idempotency String, identity "relation-only-inverse.key.v1",
    decision Target, denial RefusedBinding, handler identity "relation-only-inverse.spawn-handler.v1",
    program required, outputs SpawnOutputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Node, Facts, Id, u64, ReadOnly, NoApplicationUnit,
    field Id::reference(), value spawn_scope,
);
worth_query_mutation_binding!(pub DespawnBinding for Despawn, schema Schema,
    identity "relation-only-inverse.despawn.v1", input DespawnInput,
    operation DespawnOp, result DoneBinding,
    idempotency String, identity "relation-only-inverse.key.v1",
    decision DespawnTargets, denial RefusedBinding, handler identity "relation-only-inverse.despawn-handler.v1",
    program required, outputs DespawnOutputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Node, Facts, Id, u64, ReadOnly, NoApplicationUnit,
    field Id::reference(), value despawn_scope,
);
impl OperationCreates<SpawnOp> for Node {}
impl OperationReads<SpawnOp> for Node {}
impl OperationReads<SpawnOp> for Id {}
impl OperationReads<SpawnOp> for Holds {}
impl OperationLinks<SpawnOp> for Holds {}
impl OperationWrites<SpawnOp> for Id {}
impl ApplicationDeclaredOperation<Schema> for SpawnOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_read_entity(Self::reference(), Node::reference())
            .operation_read_field(Self::reference(), Id::reference())
            .operation_read_relation(Self::reference(), Holds::reference())
            .operation_create(Self::reference(), Node::reference())
            .operation_write(Self::reference(), Id::reference())
            .operation_link(Self::reference(), Holds::reference())
    }
}
impl OperationReads<DespawnOp> for Node {}
impl OperationReads<DespawnOp> for Id {}
impl OperationReads<DespawnOp> for Holds {}
impl OperationUnlinks<DespawnOp> for Holds {}
impl OperationDeletes<DespawnOp> for Node {}
impl ApplicationDeclaredOperation<Schema> for DespawnOp {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_read_entity(Self::reference(), Node::reference())
            .operation_read_field(Self::reference(), Id::reference())
            .operation_read_relation(Self::reference(), Holds::reference())
            .operation_unlink(Self::reference(), Holds::reference())
            .operation_delete(Self::reference(), Node::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<DespawnOp>(
            DespawnOp::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<DespawnBinding>()
}
