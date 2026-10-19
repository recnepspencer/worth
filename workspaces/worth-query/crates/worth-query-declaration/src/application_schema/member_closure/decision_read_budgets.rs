use std::collections::BTreeSet;

use super::super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

pub(super) fn validate_projection_work_budgets(
    members: &[ApplicationSchemaMember],
    operations: &BTreeSet<&str>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut limited_operations = BTreeSet::new();
    for member in members {
        let ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation,
            maximum_work_units,
        } = member
        else {
            continue;
        };
        if *maximum_work_units == 0
            || !operations.contains(operation.as_str())
            || !limited_operations.insert(operation.as_str())
        {
            return Err(ApplicationSchemaDeclarationDenial::InvalidOperationProjectionWorkBudget);
        }
    }
    Ok(())
}
