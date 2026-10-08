use sha2::{Digest, Sha256};
use worth_relational::facade::durability::{DurabilityError, RecoveryFailureClass};

use super::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes, CHECKSUM_BYTES,
    HEADER_BYTES, MAGIC,
};

mod body;
mod sizing;
#[cfg(test)]
mod tests;

impl WorthQueryApplicationCheckpoint {
    pub(in crate::domain_computation::primary_graph) fn encode(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
    ) -> Result<(Self, WorthQueryApplicationCheckpointSectionBytes), DurabilityError> {
        let native_sections = native.captured_sections().map(Into::into);
        let native_len = native.bytes().len();
        let size = sizing::ValidatedCheckpointSize::new(native_len, accepted_outputs)?;
        let mut bytes = reserve_buffer(size.total())?;
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&[0; CHECKSUM_BYTES]);
        let body_start = bytes.len();
        body::write(
            &mut bytes,
            native.bytes(),
            publication,
            accepted_outputs,
            &size,
        )?;
        // The final buffer now owns the native payload. Release its old backing
        // before hashing or any possible shrink during boxed-slice conversion.
        drop(native);
        debug_assert_eq!(bytes.len(), size.total());
        let checksum = Sha256::digest(&bytes[body_start..]);
        bytes[MAGIC.len()..body_start].copy_from_slice(&checksum);
        let sections = WorthQueryApplicationCheckpointSectionBytes::new(
            HEADER_BYTES,
            native_len,
            size.accepted(),
            accepted_outputs.len(),
            native_sections,
        );
        debug_assert_eq!(sections.total_bytes(), bytes.len());
        Ok((
            Self {
                bytes: bytes.into_boxed_slice(),
            },
            sections,
        ))
    }
}

fn reserve_buffer(total: usize) -> Result<Vec<u8>, DurabilityError> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(total).map_err(|_| {
        DurabilityError::new(
            RecoveryFailureClass::CheckpointAllocationUnavailable,
            "Query application checkpoint buffer cannot be allocated",
        )
    })?;
    Ok(bytes)
}
