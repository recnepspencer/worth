//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryProgramOutputAdvanceDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/demand/program_tree/advance.rs", "WorthQueryProgramOutputAdvanceDenial", 128 * 1024);
    }
}
