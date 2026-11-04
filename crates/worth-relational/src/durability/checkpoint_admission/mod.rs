//! Verified-original projection and protected checkpoint-root admission.
mod artifact;
mod authentication;
mod refusal;
mod wire;

pub use artifact::RelationalCheckpointAdmissionArtifact;
pub use refusal::{
    CheckpointAdmissionAuthenticationMismatch, CheckpointAdmissionBindingMismatch,
    CheckpointAdmissionMeaning, RelationalCheckpointAdmissionRefusal,
};

pub(crate) use wire::{AdmissionRowKey, EncodedAdmissionRow};

pub(crate) fn emit(
    cut: &worth_foundational::facade::CheckpointAdmissionCut,
    roots: &worth_execution::ExecutionImmutableBytes,
    rows: &[EncodedAdmissionRow],
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<RelationalCheckpointAdmissionArtifact, RelationalCheckpointAdmissionRefusal> {
    let bytes = wire::encode(cut, roots, rows, request)?;
    let length = bytes.len();
    RelationalCheckpointAdmissionArtifact::from_untrusted_region(bytes, 0..length, request)
}

pub(crate) fn capture_digest(
    bytes: &[u8],
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], crate::durability::data::RelationalNativeCheckpointCaptureDenial> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    authentication::update(&mut hash, bytes, request).map_err(|cause| {
        crate::durability::checkpoint_history::capture::capture_stop(cause.into())
    })?;
    Ok(hash.finalize().into())
}
