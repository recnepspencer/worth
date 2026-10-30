use worth_store::physical_runtime::{BlobPhysicalAllocation, RecoveryPhysicalAllocation};

fn consume_blob(_: BlobPhysicalAllocation<'_>) {}

fn substitute_recovery(allocation: RecoveryPhysicalAllocation<'_>) {
    consume_blob(allocation);
}

fn main() {}
