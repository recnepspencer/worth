//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryWorkspace>(
            "workspaces/worth-query/crates/worth-query/src/runtime/workspace.rs",
            "WorthQueryWorkspace",
            128 * 1024,
        );
        owner.field::<String>("name");
        owner.field::<WorthQueryRuntime>("runtime");
    }
}
