use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobReadLimits, PhysicalBlobFacade, RecoveryPhysicalAllocation,
};

fn substitute(
    facade: &PhysicalBlobFacade<'_>,
    recovery: RecoveryPhysicalAllocation<'_>,
    placement: AdmittedRecordPlacementPolicy,
    limits: BlobReadLimits,
) {
    let _ = facade.begin_ingest(recovery, placement, 1024, limits);
}

fn main() {}
