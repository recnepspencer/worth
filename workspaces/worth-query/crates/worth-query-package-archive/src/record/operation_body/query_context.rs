//! Both body carriage locations apply the same recursive query depth contract.
use super::{ApplicationOperationBodyProgram, Denial};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource as Source;

pub(in crate::record) fn require_nesting_depth(
    body: &ApplicationOperationBodyProgram,
    maximum_depth: u32,
) -> Result<(), Denial> {
    for operand in body
        .observation_requirements()
        .flat_map(|requirement| requirement.operands())
    {
        if let Source::QueryContext { query, .. } = operand.source() {
            crate::record::application_query::require_nesting_depth(
                query,
                maximum_depth.saturating_sub(1),
            )?;
        }
    }
    Ok(())
}
