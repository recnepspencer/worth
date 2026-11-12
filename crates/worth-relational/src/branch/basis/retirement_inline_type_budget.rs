//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<RelationalBranchBasisRetirementDenial>(
            "crates/worth-relational/src/branch/basis/retirement.rs",
            "RelationalBranchBasisRetirementDenial",
            128 * 1024,
        );
    }
}
