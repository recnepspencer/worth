use std::num::NonZeroU64;

use worth_store::physical_runtime::ServingPhysicalRuntime;

fn close_with_live_allocation(runtime: ServingPhysicalRuntime, bytes: NonZeroU64) {
    let allocation = runtime
        .physical_allocations()
        .admit_recovery(bytes)
        .unwrap();
    let _closed = runtime.close();
    drop(allocation);
}

fn main() {}
