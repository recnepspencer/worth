//! Generic local access and inherited access retain their real typed owners.
use super::*;
pub(super) trait Bound: ApplicationSchema {}
impl Bound for Schema {}
pub(super) use super::InheritedValue;

crate::worth_query_operation!(pub(super) GenericEdit for S: Bound, input Input;
    access { read_fields: [Value::at("Value")]; writes: [Value::at("Value")]; };
    inherited_access: |schema, operation| {
        schema.operation_read_field(operation, InheritedValue::at("InheritedValue"))
    };
);

#[test]
fn generic_declared_access_keeps_local_and_inherited_facts_without_an_invoice() {
    fn accepts<F: fragment::RecordSelection<Value = u64>>(_: F) {}
    accepts(Value::at("Value"));
    accepts(InheritedValue::at("InheritedValue"));
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
