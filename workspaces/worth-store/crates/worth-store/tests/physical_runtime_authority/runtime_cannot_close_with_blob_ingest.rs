use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobIngestDeclaration, BlobReadLimits, ServingPhysicalRuntime,
};

fn close_while_ingesting(
    runtime: ServingPhysicalRuntime,
    declaration: BlobIngestDeclaration,
    placement: AdmittedRecordPlacementPolicy,
    limits: BlobReadLimits,
) {
    let blobs = runtime.blobs().unwrap();
    let session = blobs
        .begin_ingest(declaration, placement, 1024, limits)
        .unwrap();
    runtime.close();
    drop(session);
}

fn main() {
    std::hint::black_box(close_while_ingesting);
}
