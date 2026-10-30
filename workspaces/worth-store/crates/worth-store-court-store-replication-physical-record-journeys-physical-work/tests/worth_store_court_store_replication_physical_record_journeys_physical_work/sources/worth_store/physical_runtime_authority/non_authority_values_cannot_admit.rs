use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, FramePortCounterObserver, ObservationHandle, PhysicalRecordId,
    PhysicalRecordInitialization, PhysicalRecordPublicationSummary, PhysicalStore,
    ProcessRuntimeCounterSnapshot, RuntimeIdentity, StoreRecordPerformanceReceipt,
};
use worth_foundational::{CanonicalDigestId, FoundationalCommitReceiptArtifact};
use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, InMemoryPhysicalFormatModel,
    InMemoryPhysicalFormatReplayArtifact, PersistedPhysicalLayout, PhysicalBinaryEncodingWitness,
};

fn admit_identity(identity: RuntimeIdentity) {
    let _runtime = PhysicalStore::admit(identity);
}

fn admit_observation(observation: ObservationHandle) {
    let _runtime = PhysicalStore::admit(observation);
}

fn admit_diagnostics(diagnostics: ProcessRuntimeCounterSnapshot) {
    let _runtime = PhysicalStore::admit(diagnostics);
}

fn admit_model(model: InMemoryPhysicalFormatModel) {
    let _runtime = PhysicalStore::admit(model);
}

fn admit_replay(replay: InMemoryPhysicalFormatReplayArtifact) {
    let _runtime = PhysicalStore::admit(replay);
}

fn admit_layout(layout: PersistedPhysicalLayout) {
    let _runtime = PhysicalStore::admit(layout);
}

fn serve_model(model: InMemoryPhysicalFormatModel, request: PhysicalRecordInitialization) {
    let _serving = model.initialize_record_store(request);
}

fn serve_replay(
    replay: InMemoryPhysicalFormatReplayArtifact,
    request: PhysicalRecordInitialization,
) {
    let _serving = replay.initialize_record_store(request);
}

fn serve_layout(layout: PersistedPhysicalLayout, request: PhysicalRecordInitialization) {
    let _serving = layout.initialize_record_store(request);
}

fn serve_backend(media: QualifiedFilesystemMedia, request: PhysicalRecordInitialization) {
    let _serving = media.initialize_record_store(request);
}

fn serve_format_witness(
    witness: PhysicalBinaryEncodingWitness,
    request: PhysicalRecordInitialization,
) {
    let _serving = witness.initialize_record_store(request);
}

fn serve_digest(digest: CanonicalDigestId, request: PhysicalRecordInitialization) {
    let _serving = digest.initialize_record_store(request);
}

fn serve_foundational_artifact(
    artifact: FoundationalCommitReceiptArtifact,
    request: PhysicalRecordInitialization,
) {
    let _serving = artifact.initialize_record_store(request);
}

fn serve_copied_identity(identity: StableStoreIdentity, request: PhysicalRecordInitialization) {
    let _serving = identity.initialize_record_store(request);
}

fn promote_weak_locator(locator: ExternalPhysicalRecordLocator) -> PhysicalRecordId {
    locator.record_id()
}

fn serve_canonical_summary(
    summary: PhysicalRecordPublicationSummary,
    request: PhysicalRecordInitialization,
) {
    let _serving = summary.initialize_record_store(request);
}

fn serve_performance_receipt(
    receipt: StoreRecordPerformanceReceipt,
    request: PhysicalRecordInitialization,
) {
    let _serving = receipt.initialize_record_store(request);
}

fn frame_counter_cannot_publish(observer: FramePortCounterObserver) {
    observer.publish_current_root();
}

fn main() {}
