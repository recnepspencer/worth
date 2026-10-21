//! Content reuse shares one initializer declaration and the ordinary native body.
use super::{
    actions::{DenialBinding, ResultBinding},
    child_actions::{Detail, DetailLabel},
    peer_audit::*,
    schema::*,
    selection_schema::*,
};
use serde::Serialize;
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_deletes, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
#[derive(Clone, Serialize)]
pub struct Select {
    pub row: String,
    pub identity: String,
    pub value: u64,
    pub note: Option<String>,
    pub label: String,
    pub row_value: u64,
}
pub struct Input;
impl ApplicationStructuredValueBinding for Input {
    type Value = Select;
    const IDENTITY_NAME: &'static str = "worth.query.reuse.input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &Select) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_operation!(pub Reuse for Schema, input Input);
worth_query_operation_reads!(Reuse=>[Row,RowIdentity,RowValue,Parent,ParentRows,ParentRevision,Snapshot,CurrentSnapshot]);
worth_query_operation_writes!(Reuse=>[RowValue,ParentRevision,SnapshotIdentity,SnapshotValue,SnapshotNote,SnapshotLabel,SnapshotKind,JournalName,JournalRevision,JournalActor,DetailLabel]);
worth_query_operation_creates!(Reuse=>[Snapshot,Journal,Detail]);
worth_query_operation_deletes!(Reuse=>[Snapshot]);
worth_query_operation_links!(Reuse=>[CurrentSnapshot,ParentJournal,JournalDetails]);
worth_query_operation_unlinks!(Reuse=>[CurrentSnapshot]);
fn scope(input: &Select) -> String {
    input.row.clone()
}
worth_query_mutation_binding!(pub Binding for Select,schema Schema,
identity "worth.query.reuse.binding.v1",input Input,operation Reuse,result ResultBinding,
idempotency u64,identity "worth.query.reuse.request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
handler identity "worth.query.reuse.body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value scope,
);
pub fn body() -> ApplicationOperationBody<Schema, Reuse, Select> {
    let op = Reuse::reference();
    let next = ApplicationOperationBodyValue::<Schema, Reuse, U64ApplicationValueBinding>::admit(
        "next",
        op.requirement_operands()
            .field("prior", RowValue::reference())
            .unwrap(),
        worth_foundational::expression_api::expressions()
            .reading_within(ApplicationOperationBodyExpression::PROFILE)
            .parse("prior + uint64(1)")
            .unwrap(),
    )
    .unwrap();
    op.body()
        .assign_value_at_path(
            ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
            ParentRevision::reference(),
            &next,
        )
        .unwrap()
        .assign_input(RowValue::reference(), "row_value")
        .unwrap()
        .create_in_scope(Snapshot::reference(), "snapshot")
        .unwrap()
        .initialize_input(SnapshotIdentity::reference(), "identity")
        .unwrap()
        .initialize_input(SnapshotValue::reference(), "value")
        .unwrap()
        .initialize_optional_input(SnapshotNote::reference(), "note")
        .unwrap()
        .initialize_input(SnapshotLabel::reference(), "label")
        .unwrap()
        .initialize(SnapshotKind::reference(), "provider".to_string())
        .unwrap()
        .replace_and_retire_from_scope(CurrentSnapshot::reference(), "current")
        .unwrap()
        .reuse_current_if_unchanged()
        .with_peer(Journal::reference(), "selection-audit", |audit| {
            audit
                .initialize_input(JournalName::reference(), "identity")?
                .initialize_value(JournalRevision::reference(), &next)?
                .initialize_from_principal_identity::<Binding, _, _, _, _, _>(
                    JournalActor::reference(),
                )?
                .with_child(JournalDetails::reference(), "audit-detail", |detail| {
                    detail.initialize(DetailLabel::reference(), "independent".into())
                })?
                .link_from_path(
                    ApplicationQueryRootPath::from(Row::reference())
                        .reverse(ParentRows::reference()),
                    ParentJournal::reference(),
                    "parent-audit",
                )
        })
        .unwrap()
        .finish()
        .unwrap()
}
#[derive(Clone, Serialize)]
pub struct EditNote {
    pub identity: String,
    pub note: Option<String>,
}
pub struct NoteInput;
impl ApplicationStructuredValueBinding for NoteInput {
    type Value = EditNote;
    const IDENTITY_NAME: &'static str = "worth.query.reuse.note-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &EditNote) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_operation!(pub ChangeNote for Schema, input NoteInput);
worth_query_operation_reads!(ChangeNote=>[Snapshot,SnapshotIdentity]);
worth_query_operation_writes!(ChangeNote=>[SnapshotNote]);
fn note_scope(input: &EditNote) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(pub NoteBinding for EditNote,schema Schema,
identity "worth.query.reuse.note-binding.v1",input NoteInput,operation ChangeNote,result ResultBinding,
idempotency u64,identity "worth.query.reuse.note-request.v1",decision WorthQueryOperationBodyDecision<Schema,Snapshot>,denial DenialBinding,
handler identity "worth.query.reuse.note-body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope Snapshot,SnapshotFacts,SnapshotIdentity,String,ReadOnly,NoApplicationUnit,field SnapshotIdentity::reference(),value note_scope,
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .operation(
            Reuse::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body())
                .unwrap(),
        )
        .operation(
            ChangeNote::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(
                    ChangeNote::reference()
                        .body()
                        .assign_optional_input(SnapshotNote::reference(), "note")
                        .unwrap()
                        .finish()
                        .unwrap(),
                )
                .unwrap(),
        )
        .operation_read_field(Reuse::reference(), RowIdentity::reference())
        .operation_read_field(ChangeNote::reference(), SnapshotIdentity::reference())
        .operation_projection_work_budget(Reuse::reference(), 256)
        .operation_projection_work_budget(ChangeNote::reference(), 64)
        .application_mutation_binding::<Binding>()
        .application_mutation_binding::<NoteBinding>()
}
