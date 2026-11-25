//! The original runtime identity a memory component was observed under.
use super::*;

pub fn require_origin(
    expected: &RelationalRuntimeOrigin,
    actual: &RelationalRuntimeOrigin,
) -> Result<(), Refusal> {
    if !expected.same_origin(actual) {
        return Err(Refusal::Basis(
            crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable,
        ));
    }
    Ok(())
}
