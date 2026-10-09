use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobIngestDeclaration, BlobIngestSession, BlobReadLimits,
    PhysicalBlobFacade,
};

fn escape<'runtime>(
    facade: PhysicalBlobFacade<'runtime>,
    declaration: BlobIngestDeclaration,
    placement: AdmittedRecordPlacementPolicy,
    limits: BlobReadLimits,
) -> BlobIngestSession<'static> {
    facade
        .begin_ingest(declaration, placement, 1024, limits)
        .unwrap()
}

fn main() {
    std::hint::black_box(escape);
}
