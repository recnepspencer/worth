//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryManagedRunTerminalKind>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/managed_run/terminal.rs", "WorthQueryManagedRunTerminalKind", 128 * 1024);
    }
}
