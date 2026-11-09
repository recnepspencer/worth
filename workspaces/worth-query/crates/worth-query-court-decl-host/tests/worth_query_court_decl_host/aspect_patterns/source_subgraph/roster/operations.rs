//! One create/retire family with member output families, using the stable source scope.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Create(pub Image);
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Restore(pub Image);
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Retire {
    pub key: String,
    pub members: Vec<String>,
}
worth_query_structured_value_binding!(pub CreateInput for Create { identity: "source-subgraph.roster-create-input.v1" });
worth_query_structured_value_binding!(pub RestoreInput for Restore { identity: "source-subgraph.roster-restore-input.v1" });
worth_query_structured_value_binding!(pub RetireInput for Retire { identity: "source-subgraph.roster-retire-input.v1" });
worth_query_operation!(pub CreateGraph for Schema, input CreateInput; access {
    read_fields: [SourceKey::at("SourceKey")];
    writes: [SourceKey::at("SourceKey"), SourceValue::at("SourceValue"), MemberKey::at("MemberKey"), Value::at("MemberValue")];
    creates: [Source, Member]; links: [OwnedBy, Next];
});
worth_query_operation!(pub RestoreGraph for Schema, input RestoreInput; access {
    read_fields: [SourceKey::at("SourceKey")];
    writes: [SourceKey::at("SourceKey"), SourceValue::at("SourceValue"), MemberKey::at("MemberKey"), Value::at("MemberValue")];
    creates: [Source, Member]; links: [OwnedBy, Next];
});
worth_query_operation!(pub RetireGraph for Schema, input RetireInput; access {
    read_fields: [SourceKey::at("SourceKey"), SourceValue::at("SourceValue"), MemberKey::at("MemberKey"), Value::at("MemberValue")];
    read_entities: [Source, Member]; read_relations: [OwnedBy, Next];
    deletes: [Source, Member]; unlinks: [OwnedBy, Next];
});
pub struct CreatedOutputs;
pub struct RetiredOutputs;
pub struct Created;
pub struct Retired;
macro_rules! root_role {
    ($role:ident, $contract:ident, $action:ty, $name:literal) => {
        impl WorthQueryApplicationOutputRole for $role {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Source;
            type Action = $action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $name;
        }
    };
}
root_role!(
    Created,
    CreatedOutputs,
    WorthQueryCreateOutput,
    "created-root"
);
root_role!(
    Retired,
    RetiredOutputs,
    WorthQueryRetireOutput,
    "retired-root"
);
pub struct CreatedMembers;
pub struct RetiredMembers;
macro_rules! member_role {
    ($role:ident, $contract:ident, $posture:ident, $prefix:literal) => {
        impl WorthQueryApplicationOutputRoleFamily for $role {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Member;
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::$posture;
            const MINIMUM: usize = 0;
        }
    };
}
member_role!(CreatedMembers, CreatedOutputs, CREATE, "created-member:");
member_role!(RetiredMembers, RetiredOutputs, RETIRE, "retired-member:");
impl ApplicationMutationOutputContract<Schema> for CreatedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Created::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[CreatedMembers::DESCRIPTOR];
}
impl ApplicationMutationOutputContract<Schema> for RetiredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Retired::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[RetiredMembers::DESCRIPTOR];
}
pub struct Retirement {
    pub root: AnchorTarget,
    pub members: Vec<(String, MemberTarget)>,
    pub links: Vec<(MemberTarget, MemberTarget)>,
}
macro_rules! binding { ($binding:ident, $input:ident, $codec:ident, $operation:ident, $decision:ty, $outputs:ident, $id:literal) => {
    fn scope(_: &$input) -> String { "source".into() }
    worth_query_mutation_binding!(pub $binding for $input, schema Schema, identity $id, input $codec, operation $operation,
        result edit::ResultBinding, idempotency String, identity $id, decision $decision, denial edit::DenialBinding,
        handler identity $id, program required, outputs $outputs,
        principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
        scope SourceKey, field SourceKey::at("SourceKey"), value scope,);
}; }
mod create {
    use super::*;
    binding!(
        CreateBinding,
        Create,
        CreateInput,
        CreateGraph,
        AnchorTarget,
        CreatedOutputs,
        "source-subgraph.roster-create.v1"
    );
}
mod restore {
    use super::*;
    binding!(
        RestoreBinding,
        Restore,
        RestoreInput,
        RestoreGraph,
        AnchorTarget,
        CreatedOutputs,
        "source-subgraph.roster-restore.v1"
    );
}
mod retire {
    use super::*;
    binding!(
        RetireBinding,
        Retire,
        RetireInput,
        RetireGraph,
        Retirement,
        RetiredOutputs,
        "source-subgraph.roster-retire.v1"
    );
}
pub use create::CreateBinding;
pub use restore::RestoreBinding;
pub use retire::RetireBinding;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .application_mutation_binding::<CreateBinding>()
        .application_mutation_binding::<RetireBinding>()
        .declared_operation::<RestoreGraph>(
            RestoreGraph::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<RestoreBinding>()
}
