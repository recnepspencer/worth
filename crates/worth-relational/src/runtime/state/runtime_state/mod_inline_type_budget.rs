//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RelationalRuntime>(
            "crates/worth-relational/src/runtime/state/runtime_state/mod.rs",
            "RelationalRuntime",
            128 * 1024,
        );
        owner.field::<std::sync::Arc<crate::runtime::RelationalRuntimeConfig>>("config");
        owner.field::<std::sync::Arc<super::super::SchemaContractRuntimeSubsystem>>(
            "schema_contract_runtime",
        );
        owner.field::<std::sync::Arc<RelationalRuntimeState>>("state");
        owner.field::<RelationalRuntimeTenure>("tenure");
        owner.field::<Option<worth_execution::ExecutionMemoryReservation>>("_outer_memory");
    }
    super::preparation_runtime::inline_type_budget::collect(census);
}
