use super::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes,
    WorthQueryCheckpointCaptureDenial, WorthQueryCheckpointCapturePolicy, CHECKSUM_BYTES,
    HEADER_BYTES, MAGIC,
};
use sha2::{Digest, Sha256};

pub(crate) mod body;
pub(crate) mod sizing;

impl WorthQueryApplicationCheckpoint {
    fn encode(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    > {
        Self::encode_frame(native, &[], &[], publication, accepted_outputs, policy)
    }
    pub(super) fn encode_frame(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        native_image: &[u8],
        world_originals: &[u8],
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    > {
        let native_sections = native.captured_sections().map(Into::into);
        let native_len = native.bytes().len();
        let size = sizing::ValidatedCheckpointSize::with_originals(
            native_len,
            native_image.len(),
            world_originals.len(),
            accepted_outputs,
            policy,
        )?;
        let mut bytes = reserve_buffer(size.total(), policy)?;
        bytes.extend_from_slice(MAGIC)?;
        bytes.extend_from_slice(&[0; CHECKSUM_BYTES])?;
        let body_start = bytes.len();
        body::write(
            &mut bytes,
            native.bytes(),
            native_image,
            world_originals,
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
            native_image.len(),
            world_originals.len(),
            size.accepted(),
            accepted_outputs.len(),
            native_sections,
        );
        debug_assert_eq!(sections.total_bytes(), bytes.len());
        Ok((
            Self {
                bytes: bytes.seal()?,
                origin: None,
            },
            sections,
        ))
    }
}

pub fn reserve_buffer(
    total: usize,
    policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
) -> Result<worth_execution::ExecutionByteBuffer, WorthQueryCheckpointCaptureDenial> {
    worth_execution::ExecutionByteBuffer::allocate(total, policy).map_err(Into::into)
}

mod court_doors_for_test;
