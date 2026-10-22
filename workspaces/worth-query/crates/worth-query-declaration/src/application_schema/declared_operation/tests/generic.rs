//! Generic local access and inherited access retain their real typed owners.
use super::*;
pub(super) trait Bound: ApplicationSchema {}
impl Bound for Schema {}
crate::worth_query_field!(pub(super) InheritedValue for S: Bound, Record, Facts:
    u64 => U64ApplicationValueBinding, read_write, equality);

crate::worth_query_operation!(pub(super) GenericEdit for S: Bound, input Input;
    access { read_fields: [Value]; writes: [Value]; };
    inherited_access: |schema, operation| {
        schema.operation_read_field(operation, InheritedValue::reference())
    };
);
impl OperationReads<GenericEdit> for InheritedValue {}

#[test]
fn generic_declared_access_keeps_local_and_inherited_facts_without_an_invoice() {
    fn accepts_local<T: OperationReads<GenericEdit> + OperationWrites<GenericEdit>>() {}
    fn accepts_inherited<T: OperationReads<GenericEdit>>() {}
    accepts_local::<Value>();
    accepts_inherited::<InheritedValue>();
    let schema = Schema::declaration().unwrap();
    let members = schema.erased().members();
    for field in ["Value", "InheritedValue"] {
        assert!(members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead {
                operation, target: ApplicationOperationDecisionReadTarget::Field { field: actual, .. }
            } if operation == "GenericEdit" && actual == field
        )));
    }
    assert!(!members.iter().any(|member| matches!(member,
        ApplicationSchemaMember::OperationProjectionWorkBudget { operation, .. } if operation == "GenericEdit"
    )));
}
