use crate::history::data::CanonicalCommitEnvelope;

use super::RelationalBranchRootCaptureDenial;
use super::RelationalRootCorrectnessIndex;
use sha2::Digest;
use std::io::Write;
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop};
use worth_relational_contracts::internal_port::durability::artifact_payload_emission::digest::Sha256Sink;

/// Typed commitment to the complete state tuple selected by a branch root.
///
/// This is descriptive integrity, never branch currentness or mutation
/// authority. It binds the truth, schema, correctness-index posture, and
/// canonical commit that readers must observe as one unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBranchVisibilityCommitment([u8; 32]);

impl RelationalBranchVisibilityCommitment {
    pub const fn from_checkpoint(digest: [u8; 32]) -> Self {
        Self(digest)
    }
    pub(super) fn for_root(
        envelope: &CanonicalCommitEnvelope,
        storage_root: [u8; 32],
        schema_root: [u8; 32],
        correctness_index: RelationalRootCorrectnessIndex,
    ) -> Self {
        compute(storage_root, schema_root, correctness_index, None, |writer| {
            envelope.write_authoritative_payload(writer)
        })
            .expect(/* Proof: The ordinary digest sink has no request refusal or I/O failure; the closed native tuple serialization is unchanged. */ "ordinary root visibility encoding")
    }

    pub(in crate::internal_port::branch) fn for_root_for_request(
        envelope: &CanonicalCommitEnvelope,
        storage_root: [u8; 32],
        schema_root: [u8; 32],
        correctness_index: RelationalRootCorrectnessIndex,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        let (result, _) = request
            .run_native_with_checkpoint(|context| {
                compute(
                    storage_root,
                    schema_root,
                    correctness_index,
                    Some(context),
                    |writer| envelope.write_authoritative_payload(writer),
                )
            })
            .map_err(|cause| RelationalBranchRootCaptureDenial::Request(cause.into()))?;
        result
    }
    pub(in crate::internal_port::branch) fn for_payload_for_request(
        payload: &[u8],
        storage_root: [u8; 32],
        schema_root: [u8; 32],
        correctness_index: RelationalRootCorrectnessIndex,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        let mut answer = None;
        let scope = request.run_native_with_checkpoint(|context| {
            answer = Some(compute(
                storage_root,
                schema_root,
                correctness_index,
                Some(context),
                |writer| writer.write_all(payload),
            ));
        });
        match (scope, answer) {
            (Ok(_), Some(answer)) => answer,
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(RelationalBranchRootCaptureDenial::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed request visibility emission"),
        }
    }

    pub const fn digest(self) -> [u8; 32] {
        self.0
    }
}

fn compute<E: std::fmt::Debug>(
    storage_root: [u8; 32],
    schema_root: [u8; 32],
    correctness_index: RelationalRootCorrectnessIndex,
    context: Option<&mut MapKernelContext<'_, '_>>,
    write_payload: impl FnOnce(&mut VisibilityWriter<'_, '_, '_>) -> Result<(), E>,
) -> Result<RelationalBranchVisibilityCommitment, RelationalBranchRootCaptureDenial> {
    let mut output = VisibilityWriter {
        hash: Sha256Sink::default(),
        context,
        stop: None,
        payload: false,
    };
    let correctness = match correctness_index {
        RelationalRootCorrectnessIndex::AuthoritativeFallback => [0],
    };
    for bytes in [
        b"worth.relational.branch-visibility.v1\0".as_slice(),
        storage_root.as_slice(),
        schema_root.as_slice(),
        correctness.as_slice(),
    ] {
        if output.write_all(bytes).is_err() {
            return Err(RelationalBranchRootCaptureDenial::Request(
                output
                    .stop
                    .expect("only original request refusal stops this sink")
                    .into(),
            ));
        }
    }
    output.payload = true;
    let encoded = write_payload(&mut output);
    if let Some(stop) = output.stop {
        return Err(RelationalBranchRootCaptureDenial::Request(stop.into()));
    }
    encoded.expect(/* Proof: The native tuple has no user serializer; the only fallible sink operation is the original request checkpoint, handled above. */ "canonical visibility encoding");
    Ok(RelationalBranchVisibilityCommitment(
        output.hash.0.finalize().into(),
    ))
}

struct VisibilityWriter<'context, 'scope, 'authority> {
    hash: Sha256Sink,
    context: Option<&'context mut MapKernelContext<'scope, 'authority>>,
    stop: Option<MapKernelStop>,
    payload: bool,
}
impl Write for VisibilityWriter<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(context) = &mut self.context {
            for chunk in bytes.chunks(64 * 1024) {
                let result = if context.has_accounting() {
                    context.checkpoint(chunk.len() as u64)
                } else {
                    context.poll()
                };
                if let Err(stop) = result {
                    self.stop = Some(stop);
                    // Other is terminal; Interrupted would make write_all retry.
                    return Err(std::io::ErrorKind::Other.into());
                }
                self.hash.write_all(chunk)?;
                if self.payload {
                    crate::execution::phase_observation::hash_bytes(
                        crate::execution::phase_observation::HashPurpose::BranchVisibility,
                        chunk.len(),
                    );
                }
            }
        } else {
            self.hash.write_all(bytes)?;
            if self.payload {
                crate::execution::phase_observation::hash_bytes(
                    crate::execution::phase_observation::HashPurpose::BranchVisibility,
                    bytes.len(),
                );
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
