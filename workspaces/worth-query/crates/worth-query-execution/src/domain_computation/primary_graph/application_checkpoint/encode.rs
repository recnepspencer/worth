use super::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes,
    WorthQueryCheckpointCaptureDenial, WorthQueryCheckpointCapturePolicy, CHECKSUM_BYTES,
    HEADER_BYTES, MAGIC,
};
use sha2::{Digest, Sha256};

mod body;
mod sizing;
#[cfg(test)]
mod tests;

impl WorthQueryApplicationCheckpoint {
    pub(in crate::domain_computation::primary_graph) fn encode(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    > {
        let native_sections = native.captured_sections().map(Into::into);
        let native_len = native.bytes().len();
        let size = sizing::ValidatedCheckpointSize::new(native_len, accepted_outputs, policy)?;
        let mut bytes = reserve_buffer(size.total(), policy)?;
        bytes.extend_from_slice(MAGIC)?;
        bytes.extend_from_slice(&[0; CHECKSUM_BYTES])?;
        let body_start = bytes.len();
        body::write(
            &mut bytes,
            native.bytes(),
            publication,
            accepted_outputs,
            &size,
        )?;
        // The final buffer now owns the native payload. Release its old backing
        // before hashing. Sealing retains this allocation without growth/shrink.
        drop(native);
        debug_assert_eq!(bytes.len(), size.total());
        let mut digest = Sha256::new();
        for chunk in bytes.bytes()[body_start..].chunks(64 * 1024) {
            bytes.check_live()?;
            digest.update(chunk);
        }
        let checksum = digest.finalize();
        bytes.overwrite(MAGIC.len(), &checksum)?;
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
                bytes: bytes.seal()?,
            },
            sections,
        ))
    }
}

fn reserve_buffer(
    total: usize,
    policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
) -> Result<worth_execution::ExecutionByteBuffer, WorthQueryCheckpointCaptureDenial> {
    worth_execution::ExecutionByteBuffer::allocate(total, policy).map_err(Into::into)
}
