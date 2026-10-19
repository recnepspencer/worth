//! The path link is the only consumer of the parent; it must carry its own observation.
use super::{
    super::{
        actions::{DenialBinding, ResultBinding},
        schema::*,
    },
    schema::*,
};
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
#[derive(Clone)]
pub struct Append {
    pub row: String,
    pub malformed_name: bool,
}
impl serde::Serialize for Append {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut record = serializer.serialize_struct("Append", 2)?;
        record.serialize_field("row", &self.row)?;
        if self.malformed_name {
            record.serialize_field("audit_name", &7_u64)?;
        } else {
            record.serialize_field("audit_name", &self.row)?;
        }
        record.end()
    }
}
worth_query_structured_value_binding!(pub Input for Append {identity:"worth.query.peer-append.input.v1",capture_input:true});
worth_query_operation!(pub AppendAudit for Schema,input Input);
worth_query_operation_reads!(AppendAudit=>[Row,RowIdentity,RowValue,Parent]);
worth_query_operation_writes!(AppendAudit=>[RowValue,AuditIdentity,AuditValue,JournalName,JournalRevision,JournalActor]);
worth_query_operation_creates!(AppendAudit=>[Audit,Journal]);
worth_query_operation_links!(AppendAudit=>[ParentJournal]);
fn scope(input: &Append) -> String {
    input.row.clone()
}
worth_query_mutation_binding!(pub Binding for Append,schema Schema,
identity "worth.query.peer-append.binding.v1",input Input,operation AppendAudit,result ResultBinding,
idempotency u64,identity "worth.query.peer-append.request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
handler identity "worth.query.peer-append.body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value scope,
candidates creates 2,deletes 0,links 1,unlinks 0,writes 6,emits 0,resources retained_representation_bytes 16384);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let body = AppendAudit::reference()
        .body()
        .assign(RowValue::reference(), 99)
        .unwrap()
        .create_in_scope(Audit::reference(), "root-audit")
        .unwrap()
        .initialize(AuditIdentity::reference(), "anchor-check".to_string())
        .unwrap()
        .initialize(AuditValue::reference(), 0)
        .unwrap()
        .with_peer(Journal::reference(), "journal", |peer| {
            peer.initialize_input(JournalName::reference(), "audit_name")?
                .initialize_from_scope(RowValue::reference(), JournalRevision::reference())?
                .initialize_from_principal_identity::<Binding, _, _, _, _, _>(
                    JournalActor::reference(),
                )?
                .link_from_path(
                    ApplicationQueryRootPath::from(Row::reference())
                        .reverse(ParentRows::reference()),
                    ParentJournal::reference(),
                    "parent-journal",
                )
        })
        .unwrap()
        .finish()
        .unwrap();
    builder
        .operation(
            AppendAudit::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation_read_field(AppendAudit::reference(), RowIdentity::reference())
        .operation_projection_work_budget(AppendAudit::reference(), 256)
        .application_mutation_binding::<Binding>()
}
