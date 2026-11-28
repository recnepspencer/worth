//! Hash the exact authoritative payload without retaining encoded body bytes.
use super::CanonicalCommitEnvelope;
use sha2::{Digest, Sha256};
use worth_relational_contracts::internal_port::durability::artifact_payload_emission::digest::{
    MeteredSha256Sink, Sha256Sink,
};

impl CanonicalCommitEnvelope {
    pub(crate) fn authoritative_payload_digest(
        &self,
    ) -> Result<[u8; 32], rmp_serde::encode::Error> {
        let mut digest = Sha256Sink::default();
        self.write_authoritative_payload(&mut digest)?;
        Ok(digest.0.finalize().into())
    }
}

impl CanonicalCommitEnvelope {
    /// Scalar observation of authoritative tuple bytes under the genuine invoking request.
    pub(crate) fn authoritative_payload_observation_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(u64, [u8; 32]), crate::durability::data::recovery_stop::RelationalRecoveryStop>
    {
        use crate::durability::data::recovery_errors::DurabilityError;
        use crate::durability::data::recovery_errors::RecoveryFailureClass;
        use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
        use worth_execution::{MapKernelStop, WorkCeilingDenial};
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let mut answer = None;
        let scope = request.run_native_with_checkpoint(|context| {
            let mut charge = |bytes| {
                if context.has_accounting() {
                    context.checkpoint(bytes as u64)
                } else {
                    context.poll()
                }
            };
            let mut writer = MeteredSha256Sink::new(Sha256::new(), &mut charge);
            let encoded = self.write_authoritative_payload(&mut writer);
            let (hash, bytes, failure) = writer.into_parts();
            answer = Some(match (encoded, failure) {
                (_, Some(cause)) => Err(Stop::Request(cause.into())),
                (Err(_), None) => Err(DurabilityError::new(
                    RecoveryFailureClass::CorruptCanonicalArtifact,
                    "canonical tuple observation refused",
                )
                .into()),
                (Ok(()), None) => Ok((bytes, hash.finalize().into())),
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
#[doc(hidden)]
pub trait CanonicalEnvelopeDigestFixturePort {
    fn authoritative_payload_digest_for_test(&self) -> Result<[u8; 32], rmp_serde::encode::Error>;
}

impl CanonicalEnvelopeDigestFixturePort for CanonicalCommitEnvelope {
    fn authoritative_payload_digest_for_test(&self) -> Result<[u8; 32], rmp_serde::encode::Error> {
        self.authoritative_payload_digest()
    }
}
