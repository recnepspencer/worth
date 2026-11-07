//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalOwnerServiceIssuanceDenial>(
            "crates/worth-signal/src/branch/owner_services/owner.rs",
            "SignalOwnerServiceIssuanceDenial",
            128 * 1024,
        );
        owner.field::<u32>("ManagedQueueStateConfigured.bound_queue_count");
        owner.field::<usize>("LiveBranchCapacityExhausted.maximum_live_branches");
        owner.field::<usize>("RetirementReceiptCapacityExhausted.maximum_retained_receipts");
    }
    {
        let mut owner = census.owner::<SignalOwner<(), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/owner.rs",
            "SignalOwner",
            128 * 1024,
        );
        owner.field::<u64>("runtime_instance_id");
        owner.field::<u64>("definition_basis");
        owner.field::<std::sync::OnceLock<recorded_initial_root::InitialEmptyRootWitness>>(
            "initial_empty_root",
        );
        owner.field::<AtomicU64>("next_branch_id");
        owner.field::<Arc<SignalOwnerLifecycleState>>("lifecycle");
        owner.field::<Arc<SignalBranchRegistry<SignalBranchCellState<(), (), ()>>>>("registry");
        owner.field::<SignalBranchBasisRegistry>("basis_registry");
        owner.field::<SignalBranchRetentionRegistry>("retention");
        owner.field::<Arc<super::super::conditional_execution::SignalConditionalRetentionLedger>>(
            "conditional_retention",
        );
        owner.field::<super::super::conditional_execution::SignalConditionalTemporalRegistry>(
            "conditional_temporal",
        );
        owner.field::<SignalBranchId>("selected_branch_id");
        owner.field::<SignalOwnerMetadata<(), (), ()>>("metadata");
        owner.field::<Arc<SignalOwnerServiceCounters>>("counters");
        owner.field::<Option<crate::durability::SignalDurableOwnerAdmission>>("_durable_admission");
    }
    super::output_retention::inline_type_budget::collect(census);
}
