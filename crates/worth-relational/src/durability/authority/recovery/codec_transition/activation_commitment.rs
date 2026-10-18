//! Bind application-owner activation admission to one exact native update.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::durability::authority::{
    DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority,
};
use crate::runtime::{
    RecoveredCheckpointProgramActivationUpdate,
    RelationalSchemaTransitionAdmissionDenial as Denial,
    RelationalSchemaTransitionAdmissionDenialKind as Kind,
};
use sha2::{Digest, Sha256};

impl DurabilityRecoveryAuthority<'_> {
    /// Compute a descriptive commitment only after admitting the exact recovered
    /// source. The application's installed owner must independently authenticate
    /// its activation protocol before issuing the concrete proof for this digest.
    pub fn checkpoint_program_activation_commitment(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovered: &RecoveredRelationalRuntimeAuthority,
        basis: &AdmittedRelationalBranchBasis,
        update: &RecoveredCheckpointProgramActivationUpdate,
    ) -> Result<[u8; 32], Denial> {
        crate::execution::check_request(execution).map_err(Denial::request)?;
        if recovered.runtime_instance_id != self.runtime.runtime_instance_id()
            || !recovered.owns_image(basis.descriptor())
            || recovered.codec_transition.is_none()
        {
            return Err(Denial::new(
                Kind::RecoveredAuthorityMismatch,
                "activation commitment requires the exact recovered codec source",
            ));
        }
        execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease,None,|context| {
                    let bytes=update.field_locator.owned_allocation_capacity_bytes()
                        .checked_add(update.expected.semantic_byte_width()).and_then(|size| size.checked_add(update.replacement.semantic_byte_width()))
                        .and_then(|size| size.checked_add(basis.descriptor().branch_id().0.len())).and_then(|size| size.checked_add(256))
                        .ok_or_else(|| Denial::request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
                    context.checkpoint(u64::try_from(bytes).map_err(|_| Denial::request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow))?).map_err(|stop| Denial::request(stop.into()))?;
                    let _memory=worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease,u64::try_from(bytes).map_err(|_| Denial::request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?)
                        .map_err(|stop| Denial::request(crate::execution::denial::lease_denial(stop)))?;
                    let locator=crate::aspect_wire::encode_aspect_field_locator(&update.field_locator);
                    let mut writer=ActivationDigest(Sha256::new());
                    writer.0.update(b"WORTH checkpoint program activation v1\0");
                    rmp_serde::encode::write(&mut writer,&(basis.descriptor(),update.entity_id,locator,&update.expected,&update.replacement))
                        .map_err(|error| Denial::new(Kind::CodecClosureRejected,error.to_string()))?;
                    Ok(writer.0.finalize().into())
                })
        }).map_err(|stop| Denial::request(stop.into()))?.map_err(|stop| Denial::request(stop.into()))?.0
    }
}

struct ActivationDigest(Sha256);
impl std::io::Write for ActivationDigest {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
