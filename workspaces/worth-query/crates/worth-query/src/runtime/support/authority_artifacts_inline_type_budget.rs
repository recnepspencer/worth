//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryRuntimeEvidenceAuthority>(
            "workspaces/worth-query/crates/worth-query/src/runtime/support/authority_artifacts.rs",
            "WorthQueryRuntimeEvidenceAuthority",
            128 * 1024,
        );
        owner.field::<()>("_private");
    }
}
