//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryApplicationProgramOutputProgress<()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/mutation/program_output_settlement.rs", "WorthQueryApplicationProgramOutputProgress", 128 * 1024);
    }
}
