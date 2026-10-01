use super::*;
use crate::record_framing::{
    encode_durable_frame_in_reserved, DURABLE_FRAME_HEADER_BYTES, FRAME_SCHEMA,
};

impl PhysicalSegmentMembershipBlock {
    pub fn encoded_frame_bytes(&self) -> Option<usize> {
        let (count, width) = match self {
            Self::Leaf { entries, .. } => (entries.len(), LEAF_ENTRY_BYTES),
            Self::Branch { children, .. } => (children.len(), REFERENCE_BYTES),
        };
        count
            .checked_mul(width)?
            .checked_add(BLOCK_PREFIX_BYTES)?
            .checked_add(DURABLE_FRAME_HEADER_BYTES)
    }

    pub fn encode_in_reserved(
        &self,
        format: PhysicalRecordFormatDeclaration,
        frame: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let (kind, count, width) = match self {
            Self::Leaf { entries, .. } => (1_u8, entries.len(), LEAF_ENTRY_BYTES),
            Self::Branch { children, .. } => (2_u8, children.len(), REFERENCE_BYTES),
        };
        let payload_bytes = self
            .encoded_frame_bytes()?
            .checked_sub(DURABLE_FRAME_HEADER_BYTES)?;
        encode_durable_frame_in_reserved(
            DurableFrameKind::SegmentMembershipBlock,
            format,
            self.block(),
            payload_bytes,
            FRAME_SCHEMA,
            frame,
            |payload| {
                payload[..8].copy_from_slice(&self.tree_identity().to_le_bytes());
                payload[8..16].copy_from_slice(&self.block().to_le_bytes());
                payload[16..18].copy_from_slice(&self.level().to_le_bytes());
                payload[18..20].copy_from_slice(&(count as u16).to_le_bytes());
                payload[20] = kind;
                payload[24..32].copy_from_slice(&self.generation().to_le_bytes());
                match self {
                    Self::Leaf { entries, .. } => {
                        for (index, entry) in entries.iter().enumerate() {
                            encode_entry(
                                &mut payload[BLOCK_PREFIX_BYTES + index * width..],
                                *entry,
                            );
                        }
                    }
                    Self::Branch { children, .. } => {
                        for (index, child) in children.iter().enumerate() {
                            encode_reference(
                                &mut payload[BLOCK_PREFIX_BYTES + index * width..],
                                *child,
                            );
                        }
                    }
                }
            },
        )
    }
}
