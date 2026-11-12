//! Real Host readers consume shared field grants; creation keeps its narrower scope.
use super::*;
struct ContentReads;
struct KeyReads;
impl OperationReads<operation::Change> for ContentReads {}
impl OperationReads<created_content::Create> for KeyReads {}

pub(super) fn change(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let scope = ApplicationDecisionReadScopeDefinition::<Schema, ContentReads>::new()
        .field(Key::at("Key"))
        .field(Values::at("Values"))
        .field(UnboundValues::at("UnboundValues"))
        .field(OtherValues::at("OtherValues"))
        .field(VersionedValues::at("VersionedValues"));
    let reference = scope.reference().expect("fixed content scope");
    schema
        .decision_read_scope(scope)
        .operation_decision_read_scope(operation::Change::reference(), reference)
}

pub(super) fn create(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let scope =
        ApplicationDecisionReadScopeDefinition::<Schema, KeyReads>::new().field(Key::at("Key"));
    let reference = scope.reference().expect("fixed key scope");
    schema
        .decision_read_scope(scope)
        .operation_decision_read_scope(created_content::Create::reference(), reference)
}
