//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryProductWorldResources>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/execution_runtime/product_world/resources.rs", "WorthQueryProductWorldResources", 128 * 1024);
        owner.field::<RuntimeWorldBudgets>("budgets");
        owner.field::<WorthQueryProductWorldClock>("clock");
        owner.field::<super::super::super::WorthQueryInvalidationResources>("invalidation");
        owner.field::<WorthQueryProductExecution>("execution");
    }
}
