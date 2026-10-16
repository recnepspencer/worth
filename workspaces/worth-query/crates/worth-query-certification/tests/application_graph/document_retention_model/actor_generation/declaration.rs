use super::super::schema::*;
use super::super::workflow::{
    WorkflowAuthoringGrant, WorkflowDocumentOwner, WorkflowGrantGrantee, WorkflowGrantGrantor,
    WorkflowGrantIdentityField,
};
use worth_query_decl::facade::{
    application_schema::*, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_deletes, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ActorLifecycleInput {
    pub identity: String,
    pub restore: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorLifecycleChanged;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActorLifecycleDenial {
    WrongState,
}
worth_query_structured_value_binding!(pub ActorLifecycleInputBinding for ActorLifecycleInput {
    identity: "worth.query.certification.actor-lifecycle-input.v1"
});
worth_query_structured_value_binding!(pub ActorLifecycleChangedBinding for ActorLifecycleChanged {
    identity: "worth.query.certification.actor-lifecycle-changed.v1"
});
worth_query_structured_value_binding!(pub ActorLifecycleDenialBinding for ActorLifecycleDenial {
    identity: "worth.query.certification.actor-lifecycle-denial.v1"
});
worth_query_operation!(pub ActorLifecycle for DocumentRetentionSchema, input ActorLifecycleInputBinding);
worth_query_operation_reads!(ActorLifecycle => [Document, Principal, ExternalMapping, WorkflowAuthoringGrant,
    DocumentIdentityField, PrincipalIdentityField, ExternalIdentityField, WorkflowGrantIdentityField,
    MappingTarget, WorkflowDocumentOwner, WorkflowGrantGrantor, WorkflowGrantGrantee]);
worth_query_operation_writes!(ActorLifecycle => [PrincipalIdentityField, MappingStatusField]);
worth_query_operation_creates!(ActorLifecycle => [Principal]);
worth_query_operation_deletes!(ActorLifecycle => [Principal]);
worth_query_operation_links!(ActorLifecycle => [MappingTarget, WorkflowDocumentOwner, WorkflowGrantGrantor, WorkflowGrantGrantee]);
worth_query_operation_unlinks!(ActorLifecycle => [MappingTarget, WorkflowDocumentOwner, WorkflowGrantGrantor, WorkflowGrantGrantee]);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let op = ActorLifecycle::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 128)
        .operation_read_entity(op, Document::reference())
        .operation_read_entity(op, Principal::reference())
        .operation_read_entity(op, ExternalMapping::reference())
        .operation_read_entity(op, WorkflowAuthoringGrant::reference())
        .operation_read_field(op, DocumentIdentityField::reference())
        .operation_read_field(op, PrincipalIdentityField::reference())
        .operation_read_field(op, ExternalIdentityField::reference())
        .operation_read_field(op, WorkflowGrantIdentityField::reference())
        .operation_read_relation(op, MappingTarget::reference())
        .operation_read_relation(op, WorkflowDocumentOwner::reference())
        .operation_read_relation(op, WorkflowGrantGrantor::reference())
        .operation_read_relation(op, WorkflowGrantGrantee::reference())
        .operation_create(op, Principal::reference())
        .operation_delete(op, Principal::reference())
        .operation_write(op, PrincipalIdentityField::reference())
        .operation_write(op, MappingStatusField::reference())
        .operation_link(op, MappingTarget::reference())
        .operation_unlink(op, MappingTarget::reference())
        .operation_link(op, WorkflowDocumentOwner::reference())
        .operation_unlink(op, WorkflowDocumentOwner::reference())
        .operation_link(op, WorkflowGrantGrantor::reference())
        .operation_unlink(op, WorkflowGrantGrantor::reference())
        .operation_link(op, WorkflowGrantGrantee::reference())
        .operation_unlink(op, WorkflowGrantGrantee::reference())
        .application_mutation_binding::<super::ActorLifecycleBinding>()
}
