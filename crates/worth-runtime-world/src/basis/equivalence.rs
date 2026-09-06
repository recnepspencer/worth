use super::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::CompositeBasisKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompositeBasisMismatch {
    expected: CompositeBasisKey,
    observed: CompositeBasisKey,
}

pub(crate) fn compare_exact(
    expected: &AdmittedCompositeRuntimeWorldBasis,
    observed: &AdmittedCompositeRuntimeWorldBasis,
) -> Result<(), CompositeBasisMismatch> {
    if expected.identity() == observed.identity() {
        Ok(())
    } else {
        Err(CompositeBasisMismatch {
            expected: expected.identity().clone(),
            observed: observed.identity().clone(),
        })
    }
}
