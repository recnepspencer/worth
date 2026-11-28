//! Incremental canonical commitment serialization under the invoking request.
use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use sha2::Sha256;
use std::io::Write;
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop, WorkCeilingDenial};
use worth_relational_contracts::internal_port::durability::artifact_payload_emission::digest::MeteredSha256Sink;

pub fn in_context<T>(
    request: &ExecutionRequest<'_, '_>,
    run: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<T, Stop>,
) -> Result<T, Stop> {
    request
        .consult()
        .map_err(|cause| Stop::Request(cause.into()))?;
    let mut result = None;
    let scope = request.run_native_with_checkpoint(|context| {
        result = Some(run(context));
    });
    match (scope, result) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Stop::Request(cause.into())),
        (Ok(_), None) => unreachable!("completed original-request commitment callback"),
    }
}

pub fn write_commitment(
    hash: Sha256,
    context: &mut MapKernelContext<'_, '_>,
    encode: impl FnOnce(&mut CommitmentWriter<'_>) -> Result<(), Stop>,
) -> Result<Sha256, Stop> {
    let mut charge = |bytes| {
        if context.has_accounting() {
            context.checkpoint(bytes as u64)
        } else {
            context.poll()
        }
    };
    let mut writer = CommitmentWriter(MeteredSha256Sink::new(hash, &mut charge));
    encode(&mut writer)?;
    Ok(writer.0.into_parts().0)
}

pub struct CommitmentWriter<'a>(MeteredSha256Sink<'a, MapKernelStop>);

impl CommitmentWriter<'_> {
    pub fn value(&mut self, value: &impl serde::Serialize) -> Result<(), Stop> {
        let encoded = rmp_serde::encode::write_named(&mut self.0, value);
        self.settle(encoded)
    }
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), Stop> {
        let encoded = self.0.write_all(bytes);
        self.settle(encoded)
    }

    fn settle(&self, encoded: Result<(), impl Sized>) -> Result<(), Stop> {
        match self.0.failure() {
            Some(cause) => Err(Stop::Request((*cause).into())),
            None => encoded.map_err(|_| corrupt()),
        }
    }
}

fn corrupt() -> Stop {
    DurabilityError::new(
        RecoveryFailureClass::CorruptCanonicalArtifact,
        "canonical commitment serializer refused admitted DATA",
    )
    .into()
}
