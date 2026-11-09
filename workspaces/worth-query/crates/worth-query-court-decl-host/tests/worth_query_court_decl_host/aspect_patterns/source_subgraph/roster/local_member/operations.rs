//! Separate lifecycle bindings expose the one member as their exact output.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Delete {
    pub owner: String,
    pub key: String,
    pub operand: String,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Restore {
    pub owner: String,
    pub key: String,
    pub operand: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub DeleteInput for Delete { identity: "source-subgraph.member-delete-input.v1" });
worth_query_structured_value_binding!(pub RestoreInput for Restore { identity: "source-subgraph.member-restore-input.v1" });
worth_query_operation!(pub DeleteMember for Schema, input DeleteInput; access {
    read_fields: [SourceKey::at("SourceKey"), MemberKey::at("MemberKey"), Value::at("MemberValue")];
    read_entities: [Source, Member]; read_relations: [OwnedBy, Head, Operand];
    deletes: [Member]; unlinks: [OwnedBy, Head, Operand];
});
worth_query_operation!(pub RestoreMember for Schema, input RestoreInput; access {
    read_fields: [SourceKey::at("SourceKey"), MemberKey::at("MemberKey")];
    read_entities: [Source, Member]; read_relations: [OwnedBy, Head, Operand];
    writes: [MemberKey::at("MemberKey"), Value::at("MemberValue")];
    creates: [Member]; links: [OwnedBy, Head, Operand];
});
pub struct Deleted;
pub struct Restored;
pub struct DeletedOutputs;
pub struct RestoredOutputs;
macro_rules! role {
    ($role:ident, $contract:ident, $action:ty, $name:literal) => {
        impl WorthQueryApplicationOutputRole for $role {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Member;
            type Action = $action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $name;
        }
    };
}
role!(
    Deleted,
    DeletedOutputs,
    WorthQueryRetireOutput,
    "deleted-member"
);
role!(
    Restored,
    RestoredOutputs,
    WorthQueryCreateOutput,
    "restored-member"
);
pub struct Additional;
impl WorthQueryApplicationOutputRoleFamily for Additional {
    type Schema = Schema;
    type Contract = RestoredOutputs;
    type Entity = Member;
    const PREFIX: &'static str = "additional-member.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
}
impl ApplicationMutationOutputContract<Schema> for DeletedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Deleted::DESCRIPTOR];
}
impl ApplicationMutationOutputContract<Schema> for RestoredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Restored::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[Additional::DESCRIPTOR];
}
pub struct Selection {
    pub owner: AnchorTarget,
    pub member: MemberTarget,
    pub operand: MemberTarget,
}
pub struct Restoration {
    pub owner: AnchorTarget,
    pub operand: MemberTarget,
}
macro_rules! binding { ($binding:ident, $input:ident, $codec:ident, $operation:ident, $decision:ty, $outputs:ident, $id:literal) => {
    fn scope(_: &$input) -> String { "source".into() }
    worth_query_mutation_binding!(pub $binding for $input, schema Schema, identity $id, input $codec, operation $operation,
        result edit::ResultBinding, idempotency String, identity $id, decision $decision, denial edit::DenialBinding,
        handler identity $id, program required, outputs $outputs,
        principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
        scope SourceKey, field SourceKey::at("SourceKey"), value scope,);
}; }
mod delete {
    use super::*;
    binding!(
        DeleteBinding,
        Delete,
        DeleteInput,
        DeleteMember,
        Selection,
        DeletedOutputs,
        "source-subgraph.member-delete.v1"
    );
}
mod restore {
    use super::*;
    binding!(
        RestoreBinding,
        Restore,
        RestoreInput,
        RestoreMember,
        Restoration,
        RestoredOutputs,
        "source-subgraph.member-restore.v1"
    );
}
pub use delete::DeleteBinding;
pub use restore::RestoreBinding;
