//! Real installed source and row mutations; no host field comparison or effect assembly.
use super::schema::*;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
};

#[derive(Clone)]
pub struct SaveRow {
    pub row: String,
    pub company: String,
    pub value: u64,
    pub calls: Option<Arc<AtomicU64>>,
}
impl Serialize for SaveRow {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let later = self
            .calls
            .as_ref()
            .map_or(0, |calls| calls.fetch_add(1, Ordering::SeqCst));
        let mut record = serializer.serialize_struct("SaveRow", 3)?;
        record.serialize_field("row", &self.row)?;
        record.serialize_field(
            "company",
            &if later == 0 {
                self.company.clone()
            } else {
                "wrong-second-encoding".into()
            },
        )?;
        record.serialize_field("value", &(self.value + later * 1000))?;
        record.end()
    }
}
pub struct SaveInput;
impl ApplicationStructuredValueBinding for SaveInput {
    type Value = SaveRow;
    const IDENTITY_NAME: &'static str = "worth.query.path-input.save.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &SaveRow) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[derive(Clone, Serialize)]
pub struct ChangeParent {
    pub parent: String,
    pub company: String,
}
#[derive(Clone, Serialize)]
pub struct IntroduceParent {
    pub row: String,
    pub identity: String,
    pub optional_company: Option<String>,
}
pub struct ChangeInput;
impl ApplicationStructuredValueBinding for ChangeInput {
    type Value = ChangeParent;
    const IDENTITY_NAME: &'static str = "worth.query.path-input.change-parent.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &ChangeParent) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub struct IntroduceInput;
impl ApplicationStructuredValueBinding for IntroduceInput {
    type Value = IntroduceParent;
    const IDENTITY_NAME: &'static str = "worth.query.path-input.introduce-parent.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &IntroduceParent) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "worth.query.path-input.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for WorthQueryOperationBodyDenial { identity: "worth.query.path-input.denial.v1" });
worth_query_operation!(pub Save for Schema, input SaveInput);
worth_query_operation!(pub Change for Schema, input ChangeInput);
worth_query_operation!(pub Introduce for Schema, input IntroduceInput);
worth_query_operation_reads!(Save => [Row, RowIdentity, Parent, ParentRows, Company]);
worth_query_operation_writes!(Save => [RowValue, AuditIdentity, AuditValue]);
worth_query_operation_creates!(Save => [Audit]);
worth_query_operation_reads!(Change => [Parent, ParentIdentity]);
worth_query_operation_writes!(Change => [Company]);
worth_query_operation_reads!(Introduce => [Row, RowIdentity]);
worth_query_operation_creates!(Introduce => [Parent]);
worth_query_operation_links!(Introduce => [ParentRows]);
worth_query_operation_writes!(Introduce => [ParentIdentity, Company, ParentRevision]);
fn row(input: &SaveRow) -> String {
    input.row.clone()
}
fn parent(input: &ChangeParent) -> String {
    input.parent.clone()
}
fn introduce_row(input: &IntroduceParent) -> String {
    input.row.clone()
}
worth_query_mutation_binding!(pub SaveBinding for SaveRow, schema Schema,
identity "worth.query.path-input.save-binding.v1", input SaveInput, operation Save, result ResultBinding,
idempotency u64, identity "worth.query.path-input.save-request.v1",
decision WorthQueryOperationBodyDecision<Schema, Row>, denial DenialBinding,
handler identity "worth.query.path-input.save-body.v1", program required, outputs NoApplicationMutationOutputs,
principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
scope Row, RowFacts, RowIdentity, String, ReadOnly, NoApplicationUnit, field RowIdentity::reference(), value row,
);
worth_query_mutation_binding!(pub ChangeBinding for ChangeParent, schema Schema,
identity "worth.query.path-input.change-binding.v1", input ChangeInput, operation Change, result ResultBinding,
idempotency u64, identity "worth.query.path-input.change-request.v1",
decision WorthQueryOperationBodyDecision<Schema, Parent>, denial DenialBinding,
handler identity "worth.query.path-input.change-body.v1", program required, outputs NoApplicationMutationOutputs,
principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
scope Parent, ParentFacts, ParentIdentity, String, ReadOnly, NoApplicationUnit, field ParentIdentity::reference(), value parent,
);
worth_query_mutation_binding!(pub IntroduceBinding for IntroduceParent, schema Schema,
identity "worth.query.path-input.introduce-binding.v1", input IntroduceInput, operation Introduce, result ResultBinding,
idempotency u64, identity "worth.query.path-input.introduce-request.v1",
decision WorthQueryOperationBodyDecision<Schema, Row>, denial DenialBinding,
handler identity "worth.query.path-input.introduce-body.v1", program required, outputs NoApplicationMutationOutputs,
principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
scope Row, RowFacts, RowIdentity, String, ReadOnly, NoApplicationUnit, field RowIdentity::reference(), value introduce_row,
);

pub fn save_body() -> ApplicationOperationBody<Schema, Save, SaveRow> {
    Save::reference()
        .body()
        .expect_input_from_path(
            ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
            Company::reference(),
            "company",
        )
        .unwrap()
        .expect_input_from_path(
            ApplicationQueryRootPath::from(Row::reference()),
            RowIdentity::reference(),
            "row",
        )
        .unwrap()
        .assign_input(RowValue::reference(), "value")
        .unwrap()
        .create_in_scope(Audit::reference(), "audit")
        .unwrap()
        .initialize_input(AuditIdentity::reference(), "row")
        .unwrap()
        .initialize_input(AuditValue::reference(), "value")
        .unwrap()
        .finish()
        .unwrap()
}
