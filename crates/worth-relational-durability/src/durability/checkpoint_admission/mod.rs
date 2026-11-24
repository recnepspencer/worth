//! Verified-original projection and protected checkpoint-root admission.
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
pub mod admit_row;
pub mod capture_base;
pub mod effect;
pub(crate) use capture_base::{
    FrozenSelectedCaptureBase, PreparedCheckpointCaptureBase, SelectedCheckpointCaptureBase,
};
pub mod artifact;
pub use branch::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
pub mod custody;
pub(crate) use custody::CheckpointAdmissionBinding;
pub use custody::CheckpointAdmissionOwner;
pub(crate) use custody::DirectoryAdmissionIdentity;
pub mod authentication;
pub mod root_positions;
pub use contracts::durability::checkpoint_admission::refusal::{
    native_stop, CheckpointAdmissionAuthenticationMismatch, CheckpointAdmissionBindingMismatch,
    CheckpointAdmissionMeaning, RelationalCheckpointAdmissionRefusal,
};
pub mod wire;

pub use artifact::RelationalCheckpointAdmissionArtifact;

pub use wire::EncodedAdmissionRow;

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
) -> Result<[u8; 32], contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial>{
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    authentication::update(&mut hash, bytes, request).map_err(|cause| {
        contracts::durability::data::native_checkpoint_capture_denial::capture_stop(cause.into())
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
