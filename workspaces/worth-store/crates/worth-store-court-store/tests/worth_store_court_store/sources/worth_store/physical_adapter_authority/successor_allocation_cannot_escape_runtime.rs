use std::num::NonZeroU64;

use worth_store::physical_runtime::{BlobPhysicalAllocation, ServingPhysicalRuntime};

fn escape_runtime(
    runtime: &ServingPhysicalRuntime,
    bytes: NonZeroU64,
) -> BlobPhysicalAllocation<'static> {
    runtime.physical_allocations().admit_blob(bytes).unwrap()
}

fn main() {}
