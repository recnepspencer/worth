//! Original-request control for the SAME positioned authoritative payload grammar.
use super::{payload_wire::ArtifactPayloadRef, RelationalCanonicalCommitArtifact as Artifact};
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalRecoveryStop as Stop,
};
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Write},
    sync::Arc,
};
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop, WorkCeilingDenial};

impl Artifact {
    pub(crate) fn matches_positioned_payload_for_request(
        original: &PositionedCanonicalCommit,
        candidate: &Arc<CanonicalCommitEnvelope>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Stop> {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let mut result = None;
        let scope = request.run_native_with_checkpoint(|context| {
            result = Some((|| {
                context
                    .poll()
                    .map_err(|cause| Stop::Request(cause.into()))?;
                if Arc::ptr_eq(original.canonical_arc(), candidate) {
                    return Ok(true);
                }
                // Both passes debit ONE original context, never a fresh ceiling.
                let expected = digest(original.position(), original.envelope(), context)?;
                let actual = digest(original.position(), candidate, context)?;
                Ok(expected == actual)
            })());
        });
        match (scope, result) {
            (Ok(_), Some(result)) => result,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(Stop::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed original-request agreement callback"),
        }
    }

    pub(crate) fn envelope_payload_digest_for_request(
        position: PatchStreamPosition,
        envelope: &CanonicalCommitEnvelope,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], Stop> {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let mut result = None;
        let scope = request.run_native_with_checkpoint(|context| {
            result = Some(digest(position, envelope, context));
        });
        match (scope, result) {
            (Ok(_), Some(result)) => result,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(Stop::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed original-request payload callback"),
        }
    }
}

fn digest(
    position: PatchStreamPosition,
    envelope: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<[u8; 32], Stop> {
    context
        .poll()
        .map_err(|cause| Stop::Request(cause.into()))?;
    let mut writer = DigestWriter {
        hash: Sha256::new(),
        context,
        stop: None,
    };
    let result =
        rmp_serde::encode::write_named(&mut writer, &ArtifactPayloadRef::new(position, envelope));
    if let Some(cause) = writer.stop {
        return Err(Stop::Request(cause.into()));
    }
    result.map_err(|_| {
        Stop::Durability(DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            "original authoritative payload serializer refused its admitted data",
        ))
    })?;
    Ok(writer.hash.finalize().into())
}
struct DigestWriter<'context, 'scope, 'authority> {
    hash: Sha256,
    context: &'context mut MapKernelContext<'scope, 'authority>,
    stop: Option<MapKernelStop>,
}
impl Write for DigestWriter<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for chunk in bytes.chunks(64 * 1024) {
            let result = if self.context.has_accounting() {
                self.context.checkpoint(chunk.len() as u64)
            } else {
                self.context.poll()
            };
            if let Err(cause) = result {
                if self.stop.is_none() {
                    self.stop = Some(cause);
                }
                return Err(io::ErrorKind::Other.into());
            }
            self.hash.update(chunk);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
