//! Separate lifecycle bindings expose every moved member through zero-minimum output families.
use super::*;
/// When `operand` is named, the first member is the roster head and links to it.
/// `revalue` writes a new value on one member that stays.
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Delete {
    pub owner: String,
    pub members: Vec<String>,
    pub operand: Option<String>,
    pub notes: Vec<String>,
    pub revalue: Option<Returned>,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Returned {
    pub key: String,
    pub value: u64,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Restore {
    pub owner: String,
    pub members: Vec<Returned>,
    pub operand: Option<String>,
    pub notes: Vec<Returned>,
    pub revalue: Option<Returned>,
}
worth_query_structured_value_binding!(pub DeleteInput for Delete { identity: "source-subgraph.member-delete-input.v1" });
worth_query_structured_value_binding!(pub RestoreInput for Restore { identity: "source-subgraph.member-restore-input.v1" });
worth_query_operation!(pub DeleteMember for Schema, input DeleteInput; access {
    read_fields: [SourceKey::at("SourceKey"), MemberKey::at("MemberKey"), Value::at("MemberValue"), NoteKey::at("NoteKey"), NoteValue::at("NoteValue")];
    read_entities: [Source, Member, Note]; read_relations: [OwnedBy, Head, Operand, NotedBy];
    writes: [Value::at("MemberValue")];
    deletes: [Member, Note]; unlinks: [OwnedBy, Head, Operand, NotedBy];
});
worth_query_operation!(pub RestoreMember for Schema, input RestoreInput; access {
    read_fields: [SourceKey::at("SourceKey"), MemberKey::at("MemberKey"), Value::at("MemberValue")];
    read_entities: [Source, Member]; read_relations: [OwnedBy, Head, Operand, NotedBy];
    writes: [MemberKey::at("MemberKey"), Value::at("MemberValue"), NoteKey::at("NoteKey"), NoteValue::at("NoteValue")];
    creates: [Member, Note]; links: [OwnedBy, Head, Operand, NotedBy];
});
pub struct Deleted;
pub struct Restored;
pub struct DeletedNotes;
pub struct RestoredNotes;
pub struct Additional;
pub struct Kept;
pub struct DeletedOutputs;
pub struct RestoredOutputs;
macro_rules! family {
    ($family:ident, $contract:ident, $entity:ty, $posture:ident, $prefix:literal) => {
        impl WorthQueryApplicationOutputRoleFamily for $family {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = $entity;
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::$posture;
            const MINIMUM: usize = 0;
        }
    };
}
family!(Deleted, DeletedOutputs, Member, RETIRE, "deleted-member.");
family!(
    Restored,
    RestoredOutputs,
    Member,
    CREATE,
    "restored-member."
);
family!(DeletedNotes, DeletedOutputs, Note, RETIRE, "deleted-note.");
family!(
    RestoredNotes,
    RestoredOutputs,
    Note,
    CREATE,
    "restored-note."
);
family!(Kept, DeletedOutputs, Member, PRESERVE, "kept-member.");
family!(
    Additional,
    RestoredOutputs,
    Member,
    CREATE,
    "additional-member."
);
impl ApplicationMutationOutputContract<Schema> for DeletedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[
        Deleted::DESCRIPTOR,
        DeletedNotes::DESCRIPTOR,
        Kept::DESCRIPTOR,
    ];
}
impl ApplicationMutationOutputContract<Schema> for RestoredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[
        Restored::DESCRIPTOR,
        RestoredNotes::DESCRIPTOR,
        Additional::DESCRIPTOR,
    ];
}
pub struct Selection {
    pub owner: AnchorTarget,
    pub members: Vec<MemberTarget>,
    pub operand: Option<MemberTarget>,
    pub notes: Vec<NoteTarget>,
    pub revalued: Option<MemberTarget>,
}
pub struct Restoration {
    pub owner: AnchorTarget,
    pub operand: Option<MemberTarget>,
    pub revalued: Option<MemberTarget>,
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
