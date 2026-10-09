use worth_relational::facade::durability::{DurabilityError, RecoveryFailureClass};

use super::super::super::{
    application_output_demand::{
        WorthQueryAcceptedOutputCheckpointIdentity, WorthQueryAcceptedOutputCheckpointPosture,
    },
    WorthQueryApplicationOutputPosture, WorthQueryPrimaryGraphPublication,
};
use super::super::{resources, WorthQueryCheckpointCaptureDenial, FORMAT_VERSION};
use super::sizing::ValidatedCheckpointSize;

pub(super) fn write(
    bytes: &mut worth_execution::ExecutionByteBuffer,
    native: &[u8],
    publication: &WorthQueryPrimaryGraphPublication,
    accepted_outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
    size: &ValidatedCheckpointSize,
) -> Result<(), WorthQueryCheckpointCaptureDenial> {
    let mut writer = ReservedFrameWriter::new(bytes, size.total())?;
    writer.bytes(&FORMAT_VERSION.to_be_bytes())?;
    writer.bytes(&publication.bootstrap_commit_id().0.to_be_bytes())?;
    writer.length(native.len())?;
    writer.length(accepted_outputs.len())?;
    writer.bytes(native)?;
    for accepted in accepted_outputs {
        writer.length(accepted.producer.len())?;
        writer.bytes(accepted.producer.as_bytes())?;
        writer.bytes(&[match accepted.posture {
            WorthQueryAcceptedOutputCheckpointPosture::Performed => 0,
            WorthQueryAcceptedOutputCheckpointPosture::StableReused => 1,
        }])?;
        writer.bytes(&accepted.source)?;
        writer.scope(accepted.scope)?;
        writer.bytes(&accepted.source_partition)?;
        writer.bytes(&[u8::from(accepted.producer_dependency.is_some())])?;
        writer.bytes(&accepted.producer_dependency.unwrap_or_default())?;
        writer.bytes(&accepted.idempotency_key)?;
        writer.bytes(&resources::encode_profile(accepted.resources))?;
        writer.length(accepted.roles.len())?;
        for role in &accepted.roles {
            writer.length(role.role.len())?;
            writer.bytes(role.role.as_bytes())?;
            writer.bytes(&[match role.posture {
                WorthQueryApplicationOutputPosture::Preserve => 0,
                WorthQueryApplicationOutputPosture::Create => 1,
                WorthQueryApplicationOutputPosture::Retire => 2,
            }])?;
            writer.length(role.entity_name.len())?;
            writer.bytes(role.entity_name.as_bytes())?;
            writer.entity(role.entity)?;
        }
        let facts = accepted.producer_facts.as_deref().unwrap_or_default();
        writer.bytes(&accepted.producer_fact_wire_version.to_be_bytes())?;
        writer.length(facts.len())?;
        writer.bytes(facts)?;
    }
    writer.finish()
}

/// Only appends inside the already reserved frame; a wire-size mismatch cannot
/// grow the fixed backing or escape as a partially encoded checkpoint.
struct ReservedFrameWriter<'a> {
    bytes: &'a mut worth_execution::ExecutionByteBuffer,
    total: usize,
}

impl<'a> ReservedFrameWriter<'a> {
    fn new(
        bytes: &'a mut worth_execution::ExecutionByteBuffer,
        total: usize,
    ) -> Result<Self, WorthQueryCheckpointCaptureDenial> {
        if bytes.capacity() != total || bytes.len() > total {
            return Err(size_mismatch().into());
        }
        Ok(Self { bytes, total })
    }

    fn bytes(&mut self, payload: &[u8]) -> Result<(), WorthQueryCheckpointCaptureDenial> {
        if payload.len() > self.total - self.bytes.len() {
            return Err(size_mismatch().into());
        }
        self.bytes.extend_from_slice(payload)?;
        Ok(())
    }

    fn length(&mut self, length: usize) -> Result<(), WorthQueryCheckpointCaptureDenial> {
        let wire = u64::try_from(length).map_err(|_| size_mismatch())?;
        self.bytes(&wire.to_be_bytes())
    }

    fn scope(
        &mut self,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) -> Result<(), WorthQueryCheckpointCaptureDenial> {
        self.bytes(&scope.partition_id().to_be_bytes())?;
        self.bytes(&scope.local_slot().to_be_bytes())?;
        self.bytes(&scope.generation().to_be_bytes())
    }

    fn entity(
        &mut self,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Result<(), WorthQueryCheckpointCaptureDenial> {
        self.bytes(&entity.partition_value().to_be_bytes())?;
        self.bytes(&entity.local_slot_value().to_be_bytes())?;
        self.bytes(&entity.generation_value().to_be_bytes())
    }

    fn finish(self) -> Result<(), WorthQueryCheckpointCaptureDenial> {
        if self.bytes.len() == self.total {
            Ok(())
        } else {
            Err(size_mismatch().into())
        }
    }
}

fn size_mismatch() -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::CheckpointFrameSizeMismatch,
        "Query application checkpoint emission differs from its reserved frame size",
    )
}
