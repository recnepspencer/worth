//! Both replacement postures run installed native bodies, never test effect builders.
use super::{
    actions::{DenialBinding, ResultBinding},
    schema::*,
    selection_schema::*,
};
use serde::Serialize;
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_deletes, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
#[derive(Clone, Serialize)]
pub struct KeepSnapshot {
    pub row: String,
    pub identity: String,
    pub value: u64,
}
#[derive(Clone, Serialize)]
pub struct RetireSnapshot {
    pub row: String,
    pub identity: String,
    pub value: u64,
}
pub struct KeepInput;
pub struct RetireInput;
impl ApplicationStructuredValueBinding for KeepInput {
    type Value = KeepSnapshot;
    const IDENTITY_NAME: &'static str = "worth.query.retiring.keep-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &KeepSnapshot) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
impl ApplicationStructuredValueBinding for RetireInput {
    type Value = RetireSnapshot;
    const IDENTITY_NAME: &'static str = "worth.query.retiring.retire-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &RetireSnapshot) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_operation!(pub Keep for Schema, input KeepInput);
worth_query_operation!(pub Retire for Schema, input RetireInput);
worth_query_operation_reads!(Keep=>[Row,RowIdentity,Snapshot,CurrentSnapshot]);
worth_query_operation_reads!(Retire=>[Row,RowIdentity,Snapshot,CurrentSnapshot]);
worth_query_operation_writes!(Keep=>[RowValue,SnapshotIdentity,SnapshotValue]);
worth_query_operation_writes!(Retire=>[RowValue,SnapshotIdentity,SnapshotValue]);
worth_query_operation_creates!(Keep=>[Snapshot]);
worth_query_operation_creates!(Retire=>[Snapshot]);
worth_query_operation_deletes!(Retire=>[Snapshot]);
worth_query_operation_links!(Keep=>[CurrentSnapshot]);
worth_query_operation_links!(Retire=>[CurrentSnapshot]);
worth_query_operation_unlinks!(Keep=>[CurrentSnapshot]);
worth_query_operation_unlinks!(Retire=>[CurrentSnapshot]);
fn keep_row(value: &KeepSnapshot) -> String {
    value.row.clone()
}
fn retire_row(value: &RetireSnapshot) -> String {
    value.row.clone()
}
worth_query_mutation_binding!(pub KeepBinding for KeepSnapshot,schema Schema,
    identity "worth.query.retiring.keep-binding.v1", input KeepInput,operation Keep,result ResultBinding,
    idempotency u64,identity "worth.query.retiring.keep-request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
    handler identity "worth.query.retiring.keep-body.v1",program required,outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value keep_row,
    candidates creates 1,deletes 0,links 1,unlinks 1,writes 3,emits 0,resources retained_representation_bytes 4096);
worth_query_mutation_binding!(pub RetireBinding for RetireSnapshot,schema Schema,
    identity "worth.query.retiring.retire-binding.v1", input RetireInput,operation Retire,result ResultBinding,
    idempotency u64,identity "worth.query.retiring.retire-request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
    handler identity "worth.query.retiring.retire-body.v1",program required,outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value retire_row,
    candidates creates 1,deletes 1,links 1,unlinks 1,writes 3,emits 0,resources retained_representation_bytes 4096);
pub fn keep_body() -> ApplicationOperationBody<Schema, Keep, KeepSnapshot> {
    Keep::reference()
        .body()
        .in_scope(Row::reference())
        .unwrap()
        .assign_input(RowValue::reference(), "value")
        .unwrap()
        .create_in_scope(Snapshot::reference(), "snapshot")
        .unwrap()
        .initialize_input(SnapshotIdentity::reference(), "identity")
        .unwrap()
        .initialize_input(SnapshotValue::reference(), "value")
        .unwrap()
        .replace_from_scope(CurrentSnapshot::reference(), "current")
        .unwrap()
        .finish()
        .unwrap()
}
pub fn retire_body() -> ApplicationOperationBody<Schema, Retire, RetireSnapshot> {
    Retire::reference()
        .body()
        .in_scope(Row::reference())
        .unwrap()
        .assign_input(RowValue::reference(), "value")
        .unwrap()
        .create_in_scope(Snapshot::reference(), "snapshot")
        .unwrap()
        .initialize_input(SnapshotIdentity::reference(), "identity")
        .unwrap()
        .initialize_input(SnapshotValue::reference(), "value")
        .unwrap()
        .replace_and_retire_from_scope(CurrentSnapshot::reference(), "current")
        .unwrap()
        .finish()
        .unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .operation(
            Keep::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(keep_body())
                .unwrap(),
        )
        .operation(
            Retire::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(retire_body())
                .unwrap(),
        )
        .operation_read_field(Keep::reference(), RowIdentity::reference())
        .operation_read_field(Retire::reference(), RowIdentity::reference())
        .operation_projection_work_budget(Keep::reference(), 128)
        .operation_projection_work_budget(Retire::reference(), 128)
        .application_mutation_binding::<KeepBinding>()
        .application_mutation_binding::<RetireBinding>()
}
