//! Verified-original projection and protected checkpoint-root admission.
mod admitted_commit;
mod capture_base;
pub(crate) use capture_base::{
    FrozenSelectedCaptureBase, PreparedCheckpointCaptureBase, SelectedCheckpointCaptureBase,
};
mod artifact;
pub(crate) use admitted_commit::CheckpointAdmittedCommit;
mod custody;
pub(crate) use custody::{AdmissionCustody, CheckpointAdmissionBinding, CheckpointAdmissionOwner};
mod authentication;
pub(crate) mod binary_encoding;
mod refusal;
mod root_positions;
pub(crate) use refusal::native_stop;
pub(crate) use root_positions::projected_stop;
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
    let (bytes, directory) = wire::encode(cut, roots, rows, request)?;
    RelationalCheckpointAdmissionArtifact::from_emission(bytes, directory, request)
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

pub(crate) fn binding_digest(
    bytes: &[u8],
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], RelationalCheckpointAdmissionRefusal> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    authentication::update(&mut hash, bytes, request)?;
    Ok(hash.finalize().into())
}
