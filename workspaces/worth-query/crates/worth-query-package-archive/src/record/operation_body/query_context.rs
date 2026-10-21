//! Both body carriage locations apply the same recursive query depth contract.
use super::{ApplicationOperationBodyProgram, Denial};

pub(in crate::record) fn require_nesting_depth(
    body: &ApplicationOperationBodyProgram,
    maximum_depth: u32,
) -> Result<(), Denial> {
    for (_, query) in body.query_contexts() {
        crate::record::application_query::require_nesting_depth(
            query,
            maximum_depth.saturating_sub(1),
        )?;
    }
    Ok(())
}
