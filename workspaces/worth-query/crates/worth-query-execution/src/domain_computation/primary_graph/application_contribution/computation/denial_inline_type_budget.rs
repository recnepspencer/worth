//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryManagedComputationResourceDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/computation/denial.rs", "WorthQueryManagedComputationResourceDenial", 128 * 1024);
        owner.field::<worth_execution::ExecutionPhysicalAllocationDenialKind>(
            "NativeAllocationRefused.kind",
        );
        owner.field::<Option<u64>>("NativeAllocationRefused.requested_payload_bytes");
        owner.field::<u64>("MemoryLimit.requested");
        owner.field::<u64>("MemoryLimit.admitted");
        owner.field::<WorthQueryMemoryLimitLevel>("MemoryLimit.level");
    }
    {
        let _owner = census.owner::<WorthQueryManagedComputationInterruption>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/computation/denial.rs", "WorthQueryManagedComputationInterruption", 128 * 1024);
    }
}
