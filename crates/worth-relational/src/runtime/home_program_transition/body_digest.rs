//! Versioned commitment to authored inputs; it confers no execution authority.
use crate::aspect_wire::CanonicalAspectOutput;
use crate::mvcc::RelationalTransactionIntent;
use crate::transactions::data::WorkerIntentBatch;
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::HomeProgramTransitionBodyDigest;

mod mutations;
mod patches;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum RelationalHomeProgramBodyDenial {
    Request(WorkCeilingDenial),
    CapacityOverflow,
    Encoding,
    UnsupportedMaterialization,
}

/// Hash the complete ordered input before any demand, normalization or rule.
pub fn home_program_transition_body_digest(
    request: &ExecutionRequest<'_, '_>,
    intent: &RelationalTransactionIntent,
    batches: &[WorkerIntentBatch],
) -> Result<HomeProgramTransitionBodyDigest, RelationalHomeProgramBodyDenial> {
    use RelationalHomeProgramBodyDenial as Denial;
    request.consult().map_err(Denial::Request)?;
    let (savepoints, transition, policy) = intent
        .home_authored_policy()
        .ok_or(Denial::UnsupportedMaterialization)?;
    let mut out = BodyOutput {
        request,
        hash: Sha256::new(),
        stopped: None,
    };
    out.bytes(b"worth.native.home.authored-body.v1\0")?;
    out.byte(u8::from(savepoints))?;
    out.scalar(&transition)?;
    out.scalar(&policy)?;
    out.length(batches.len())?;
    for WorkerIntentBatch {
        name,
        partition_key,
        worker_local_only,
        intents,
    } in batches
    {
        out.text(name)?;
        match partition_key {
            None => out.byte(0)?,
            Some(key) => {
                out.byte(1)?;
                out.text(key)?;
            }
        }
        out.byte(u8::from(*worker_local_only))?;
        out.length(intents.len())?;
        for intent in intents {
            mutations::write(&mut out, intent)?;
        }
    }
    Ok(HomeProgramTransitionBodyDigest::from_recorded(
        out.hash.finalize().into(),
    ))
}

struct BodyOutput<'request, 'control, 'scope> {
    request: &'request ExecutionRequest<'control, 'scope>,
    hash: Sha256,
    stopped: Option<RelationalHomeProgramBodyDenial>,
}
impl BodyOutput<'_, '_, '_> {
    fn scalar(
        &mut self,
        value: &impl serde::Serialize,
    ) -> Result<(), RelationalHomeProgramBodyDenial> {
        serde_json::to_writer(&mut *self, value).map_err(|_| {
            self.stopped
                .take()
                .unwrap_or(RelationalHomeProgramBodyDenial::Encoding)
        })?;
        // JSON escapes NUL, so scalar/schema values remain self-delimiting.
        // Its string escape scan precedes output; chunk polling bounds the
        // sink's hashing work, not that serializer-internal scan.
        self.byte(0)
    }
}
impl CanonicalAspectOutput for BodyOutput<'_, '_, '_> {
    type Error = RelationalHomeProgramBodyDenial;
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        // Empty leaves still observe the original request.
        self.request.consult().map_err(Self::Error::Request)?;
        for chunk in bytes.chunks(4096) {
            self.request.consult().map_err(Self::Error::Request)?;
            self.request
                .run_native_with_checkpoint(|context| {
                    if context.has_accounting() {
                        context.checkpoint(chunk.len() as u64)
                    } else {
                        context.poll()
                    }
                })
                .and_then(|(answer, _)| answer.map_err(WorkCeilingDenial::Stopped))
                .map_err(Self::Error::Request)?;
            self.hash.update(chunk);
        }
        Ok(())
    }
    fn length(&mut self, length: usize) -> Result<(), Self::Error> {
        let length = u32::try_from(length).map_err(|_| Self::Error::CapacityOverflow)?;
        self.bytes(&length.to_le_bytes())
    }
}
impl std::io::Write for BodyOutput<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match CanonicalAspectOutput::bytes(self, bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(cause) => {
                self.stopped = Some(cause);
                Err(std::io::Error::other("Home body request refused"))
            }
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
