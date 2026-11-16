//! A self-inverse revision: a preserved owner beside two zero-minimum member families.
use super::*;
use local_member::Returned;
/// `value` rewrites the owner's value; `mark` edits its optional mark;
/// `retire` names members that leave and `create` the members that arrive.
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Revise {
    pub owner: String,
    pub value: Option<u64>,
    pub mark: MarkEdit,
    pub retire: Vec<String>,
    pub create: Vec<Returned>,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum MarkEdit {
    Unchanged,
    Clear,
    Set(u64),
}
worth_query_structured_value_binding!(pub ReviseInput for Revise { identity: "source-subgraph.owner-revise-input.v1" });
worth_query_operation!(pub ReviseOwner for Schema, input ReviseInput);
worth_query_operation_reads!(ReviseOwner => [Source, Member]);
worth_query_operation_creates!(ReviseOwner => [Member]);
worth_query_operation_deletes!(ReviseOwner => [Member]);
worth_query_operation_links!(ReviseOwner => [OwnedBy]);
worth_query_operation_unlinks!(ReviseOwner => [OwnedBy]);
worth_query_operation_removes_aspects!(ReviseOwner => [SourceMark]);
impl OperationReads<ReviseOwner> for OwnedBy {}
impl ApplicationDeclaredOperation<Schema> for ReviseOwner {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        let operation = Self::reference();
        schema
            .operation_read_entity(operation, Source::reference())
            .operation_read_entity(operation, Member::reference())
            .operation_read_field(operation, SourceKey::at("SourceKey"))
            .operation_read_field(operation, SourceValue::at("SourceValue"))
            .operation_read_field(operation, OwnerMark::at("OwnerMark"))
            .operation_read_field(operation, MemberKey::at("MemberKey"))
            .operation_read_field(operation, Value::at("MemberValue"))
            .operation_read_relation(operation, OwnedBy::reference())
            .operation_write(operation, SourceValue::at("SourceValue"))
            .operation_write(operation, OwnerMark::at("OwnerMark"))
            .operation_write(operation, MemberKey::at("MemberKey"))
            .operation_write(operation, Value::at("MemberValue"))
            .operation_remove_aspect(operation, SourceMark::reference())
            .operation_create(operation, Member::reference())
            .operation_delete(operation, Member::reference())
            .operation_link(operation, OwnedBy::reference())
            .operation_unlink(operation, OwnedBy::reference())
    }
}
pub struct RevisedOwner;
impl WorthQueryApplicationOutputRole for RevisedOwner {
    type Schema = Schema;
    type Contract = RevisionOutputs;
    type Entity = Source;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "revised-owner";
}
pub struct LeavingMembers;
pub struct ArrivingMembers;
pub struct RevisionOutputs;
macro_rules! family {
    ($family:ident, $posture:ident, $prefix:literal) => {
        impl WorthQueryApplicationOutputRoleFamily for $family {
            type Schema = Schema;
            type Contract = RevisionOutputs;
            type Entity = Member;
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::$posture;
            const MINIMUM: usize = 0;
        }
    };
}
family!(LeavingMembers, RETIRE, "leaving-member.");
family!(ArrivingMembers, CREATE, "arriving-member.");
impl ApplicationMutationOutputContract<Schema> for RevisionOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[RevisedOwner::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[LeavingMembers::DESCRIPTOR, ArrivingMembers::DESCRIPTOR];
}
pub struct Revised {
    pub owner: AnchorTarget,
    pub leaving: Vec<MemberTarget>,
}
fn scope(_: &Revise) -> String {
    "source".into()
}
worth_query_mutation_binding!(pub ReviseBinding for Revise, schema Schema,
    identity "source-subgraph.owner-revise.v1", input ReviseInput, operation ReviseOwner,
    result edit::ResultBinding, idempotency String, identity "source-subgraph.owner-revise.v1",
    decision Revised, denial edit::DenialBinding,
    handler identity "source-subgraph.owner-revise.v1",
    program required, outputs RevisionOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope SourceKey, field SourceKey::at("SourceKey"), value scope,);
