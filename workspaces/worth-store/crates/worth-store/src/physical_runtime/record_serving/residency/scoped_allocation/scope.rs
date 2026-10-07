use worth_store_buffer_pool::PhysicalOperationAllocationScope;

pub(super) trait StoreAllocationScope {
    const VALUE: PhysicalOperationAllocationScope;
}

macro_rules! store_allocation_scope {
    ($name:ident, $value:ident) => {
        pub(super) enum $name {}

        impl StoreAllocationScope for $name {
            const VALUE: PhysicalOperationAllocationScope =
                PhysicalOperationAllocationScope::$value;
        }
    };
}

store_allocation_scope!(RecoveryScope, Recovery);
store_allocation_scope!(ScrubScope, Scrub);
store_allocation_scope!(MaintenanceScope, Maintenance);
store_allocation_scope!(VerificationScope, Verification);
store_allocation_scope!(BlobScope, Blob);
store_allocation_scope!(LayoutReadScope, ForegroundRead);
