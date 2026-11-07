//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryProgramApplicationRuntime<(), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/program_runtime.rs", "WorthQueryProgramApplicationRuntime", 128 * 1024);
        owner.field::<std::sync::Arc<WorthQueryProgramRuntimeBacking<(), ()>>>("backing");
    }
}
