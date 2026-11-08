//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryApplicationRequestMutationDenialKind>(
            "workspaces/worth-query/crates/worth-query-publication/src/application_entry/denial.rs",
            "WorthQueryApplicationRequestMutationDenialKind",
            128 * 1024,
        );
    }
    {
        let _owner = census.owner::<WorthQueryApplicationRequestMutationDenial>(
            "workspaces/worth-query/crates/worth-query-publication/src/application_entry/denial.rs",
            "WorthQueryApplicationRequestMutationDenial",
            128 * 1024,
        );
    }
}
