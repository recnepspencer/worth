use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedRecordPlacementPolicy, BlobIngestDeclaration, BlobIngestFailure,
    BlobIngestSession, BlobReadLimits, BlobReclaimContinuationFailure, BlobReclaimFailure,
    BlobReclaimLimits, BlobReclaimReceipt, BlobReclaimRequest, BlobReclaimRetirement,
    BlobReclaimRetirementBudget, BlobResumeFailure, BlobResumeLimits, BlobResumeToken,
    BlobTerminalFailure, BlobTerminalLimits, BlobTerminalReceipt, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

fn begin<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    declaration: BlobIngestDeclaration,
    placement: AdmittedRecordPlacementPolicy,
    limits: BlobReadLimits,
) -> Result<BlobIngestSession<'runtime>, BlobIngestFailure> {
    runtime
        .blobs()
        .unwrap()
        .begin_ingest(declaration, placement, 1024, limits)
}

fn close_after_ingest_releases_its_authority(
    runtime: ServingPhysicalRuntime,
    declaration: BlobIngestDeclaration,
    placement: AdmittedRecordPlacementPolicy,
    limits: BlobReadLimits,
) {
    let session = begin(&runtime, declaration, placement, limits).unwrap();
    drop(session);
    runtime.close();
}

fn resume<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    limits: BlobResumeLimits,
) -> Result<BlobIngestSession<'runtime>, BlobResumeFailure> {
    runtime
        .blobs()
        .unwrap()
        .resume_ingest(token, scope, placement, 1024, deadline, limits)
}

fn abort(
    runtime: &ServingPhysicalRuntime,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    limits: BlobTerminalLimits,
) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
    runtime
        .blobs()
        .unwrap()
        .abort_ingest(token, scope, placement, deadline, limits)
}

fn expire(
    runtime: &ServingPhysicalRuntime,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    limits: BlobTerminalLimits,
) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
    runtime
        .blobs()
        .unwrap()
        .expire_ingest(token, scope, placement, deadline, limits)
}

fn reclaim_abandoned(
    runtime: &ServingPhysicalRuntime,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    limits: BlobReclaimLimits,
) -> Result<BlobReclaimReceipt, BlobReclaimFailure> {
    runtime
        .blobs()
        .unwrap()
        .reclaim(BlobReclaimRequest::abandoned(
            token, scope, placement, deadline, limits,
        ))?
        .wait()
}

fn continue_reclaim_retirement(
    runtime: &ServingPhysicalRuntime,
    receipt: &mut BlobReclaimReceipt,
    budget: BlobReclaimRetirementBudget,
) -> Result<BlobReclaimRetirement, BlobReclaimContinuationFailure> {
    runtime
        .blobs()
        .unwrap()
        .continue_reclaim_retirement(receipt, budget)
}

fn main() {
    std::hint::black_box(begin);
    std::hint::black_box(resume);
    std::hint::black_box(abort);
    std::hint::black_box(expire);
    std::hint::black_box(reclaim_abandoned);
    std::hint::black_box(continue_reclaim_retirement);
    std::hint::black_box(close_after_ingest_releases_its_authority);
}
