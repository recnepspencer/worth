//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryWorkspaceError>(
            "workspaces/worth-query/crates/worth-query/src/memory_workspace/mod.rs",
            "WorthQueryWorkspaceError",
            128 * 1024,
        );
        owner.field::<WorthQueryWorkspaceErrorKind>("kind");
        owner.field::<String>("message");
    }
}
