//! Domain-separated commitments to actual projected bytes under the common cut.
use super::{
    wire::AdmissionRowKey, CheckpointAdmissionMeaning,
    RelationalCheckpointAdmissionRefusal as Refusal,
};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

pub(super) fn meaning(
    cut: &[u8],
    key: AdmissionRowKey,
    role: CheckpointAdmissionMeaning,
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], Refusal> {
    let mut hash = Sha256::new();
    update(
        &mut hash,
        b"worth.relational.checkpoint-admission.meaning.v1\0",
        request,
    )?;
    update(
        &mut hash,
        &super::wire::VERSION.get().to_le_bytes(),
        request,
    )?;
    update(&mut hash, &(cut.len() as u64).to_le_bytes(), request)?;
    update(&mut hash, cut, request)?;
    update(&mut hash, &key.position.to_le_bytes(), request)?;
    update(&mut hash, &key.commit.to_le_bytes(), request)?;
    update(&mut hash, &[role as u8], request)?;
    update(&mut hash, &(payload.len() as u64).to_le_bytes(), request)?;
    update(&mut hash, payload, request)?;
    Ok(hash.finalize().into())
}

pub(super) fn directory(
    header: &[u8],
    cut: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Sha256, Refusal> {
    let mut hash = Sha256::new();
    update(
        &mut hash,
        b"worth.relational.checkpoint-admission.directory.v1\0",
        request,
    )?;
    update(&mut hash, header, request)?;
    update(&mut hash, cut, request)?;
    Ok(hash)
}

pub(super) fn update(
    hash: &mut Sha256,
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let (result, _) = request
        .run_native_with_checkpoint(|context| {
            for chunk in bytes.chunks(64 * 1024) {
                if context.has_accounting() {
                    context.checkpoint(chunk.len() as u64)?;
                } else {
                    context.poll()?;
                }
                hash.update(chunk);
            }
            Ok(())
        })
        .map_err(|cause| Refusal::Request(cause.into()))?;
    result.map_err(|cause| Refusal::Request(WorkCeilingDenial::Stopped(cause).into()))
}
