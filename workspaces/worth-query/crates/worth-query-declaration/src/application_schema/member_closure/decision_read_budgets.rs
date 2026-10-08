use std::collections::BTreeSet;

use super::super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

pub(super) fn validate_projection_work_budgets(
    members: &[ApplicationSchemaMember],
    operations: &BTreeSet<&str>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let read_operations = members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationDecisionRead { operation, .. } => {
                Some(operation.as_str())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for operation in read_operations {
        let projection_budgets = members
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::OperationProjectionWorkBudget {
                    operation: candidate,
                    maximum_work_units,
                } if candidate == operation => Some(*maximum_work_units),
                _ => None,
            })
            .collect::<Vec<_>>();
        if projection_budgets.len() != 1
            || projection_budgets[0] == 0
            || !operations.contains(operation)
        {
            return Err(ApplicationSchemaDeclarationDenial::InvalidOperationProjectionWorkBudget);
        }
    }
    Ok(())
}
