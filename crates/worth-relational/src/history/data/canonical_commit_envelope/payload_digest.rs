//! Hash the exact authoritative payload without retaining encoded body bytes.
use super::CanonicalCommitEnvelope;
use sha2::{Digest, Sha256};
use std::io::{Result as IoResult, Write};

struct PayloadDigest {
    hash: Sha256,
    bytes: u64,
}
impl Write for PayloadDigest {
    fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("canonical payload byte count overflow"))?;
        self.hash.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}
impl CanonicalCommitEnvelope {
    pub(crate) fn authoritative_payload_digest(
        &self,
    ) -> Result<[u8; 32], rmp_serde::encode::Error> {
        self.authoritative_payload_observation()
            .map(|(_, digest)| digest)
    }
    pub(crate) fn authoritative_payload_observation(
        &self,
    ) -> Result<(u64, [u8; 32]), rmp_serde::encode::Error> {
        let mut digest = PayloadDigest {
            hash: Sha256::new(),
            bytes: 0,
        };
        self.write_authoritative_payload(&mut digest)?;
        Ok((digest.bytes, digest.hash.finalize().into()))
    }
}

impl CanonicalCommitEnvelope {
    /// Scalar observation of authoritative tuple bytes under the genuine invoking request.
    pub(crate) fn authoritative_payload_observation_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(u64, [u8; 32]), crate::durability::data::RelationalRecoveryStop> {
        use crate::durability::data::{
            DurabilityError, RecoveryFailureClass, RelationalRecoveryStop as Stop,
        };
        use worth_execution::{MapKernelStop, WorkCeilingDenial};
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let mut answer = None;
        let scope = request.run_native_with_checkpoint(|context| {
            let mut writer = RequestPayloadDigest {
                inner: PayloadDigest {
                    hash: Sha256::new(),
                    bytes: 0,
                },
                context,
                stop: None,
            };
            let encoded = self.write_authoritative_payload(&mut writer);
            answer = Some(match (encoded, writer.stop.take()) {
                (_, Some(cause)) => Err(Stop::Request(cause.into())),
                (Err(_), None) => Err(DurabilityError::new(
                    RecoveryFailureClass::CorruptCanonicalArtifact,
                    "canonical tuple observation refused",
                )
                .into()),
                (Ok(()), None) => Ok((writer.inner.bytes, writer.inner.hash.finalize().into())),
            });
        });
        match (scope, answer) {
            (Ok(_), Some(answer)) => answer,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(Stop::Request(cause.into())),
            (Ok(_), None) => unreachable!("completed scalar tuple observation"),
        }
    }
}
struct RequestPayloadDigest<'c, 's, 'a> {
    inner: PayloadDigest,
    context: &'c mut worth_execution::MapKernelContext<'s, 'a>,
    stop: Option<worth_execution::MapKernelStop>,
}
impl Write for RequestPayloadDigest<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
        for chunk in bytes.chunks(64 * 1024) {
            let result = if self.context.has_accounting() {
                self.context.checkpoint(chunk.len() as u64)
            } else {
                self.context.poll()
            };
            if let Err(cause) = result {
                self.stop = Some(cause);
                return Err(std::io::Error::other(
                    "canonical observation request refused",
                ));
            }
            self.inner.write_all(chunk)?;
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}
