//! Canonical payload projection; framing and integrity admission belong to callers.
use super::*;

impl PhysicalSegmentMembershipBlock {
    /// Projects this family's payload without inspecting or authenticating a durable frame.
    /// This pure format operation grants no persisted-source or decoder authority.
    pub fn project_payload(
        payload: &[u8],
        block_identity: u64,
        capacity: u16,
        limits: SegmentMembershipBlockDecodeLimits,
    ) -> Result<Self, BoundedSegmentMembershipBlockDecodeDenial> {
        if payload.len() < BLOCK_PREFIX_BYTES
            || payload[21..24] != [0; 3]
            || payload[32..40] != [0; 8]
        {
            return Err(SegmentMembershipBlockDenial::MalformedPrefix.into());
        }
        let tree_identity = u64::from_le_bytes(payload[..8].try_into().unwrap());
        let block = u64::from_le_bytes(payload[8..16].try_into().unwrap());
        let level = u16::from_le_bytes(payload[16..18].try_into().unwrap());
        let count = u16::from_le_bytes(payload[18..20].try_into().unwrap());
        let generation = u64::from_le_bytes(payload[24..32].try_into().unwrap());
        if tree_identity == 0
            || generation == 0
            || block == 0
            || block != block_identity
            || count == 0
            || count > capacity
        {
            return Err(SegmentMembershipBlockDenial::IdentityOrCapacity.into());
        }
        let entry_bytes = match payload[20] {
            1 if level == 0 => LEAF_ENTRY_BYTES,
            2 if level != 0 => REFERENCE_BYTES,
            _ => return Err(SegmentMembershipBlockDenial::LevelOrKind.into()),
        };
        if payload.len() != BLOCK_PREFIX_BYTES + usize::from(count) * entry_bytes {
            return Err(SegmentMembershipBlockDenial::MalformedLength.into());
        }
        let observed = u64::from(count);
        if level == 0 && observed > limits.leaf_entries {
            return Err(BoundedSegmentMembershipBlockDecodeDenial::LeafEntries {
                observed,
                admitted: limits.leaf_entries,
            });
        }
        if level != 0 && observed > limits.branch_children {
            return Err(BoundedSegmentMembershipBlockDecodeDenial::BranchChildren {
                observed,
                admitted: limits.branch_children,
            });
        }
        let body = &payload[BLOCK_PREFIX_BYTES..];
        let decoded = if level == 0 {
            let entries = body
                .chunks_exact(entry_bytes)
                .map(decode_entry)
                .collect::<Option<Vec<_>>>()
                .ok_or(SegmentMembershipBlockDenial::InvalidEntry)?;
            Self::leaf(tree_identity, generation, block, entries, capacity)
        } else {
            let children = body
                .chunks_exact(entry_bytes)
                .map(decode_reference)
                .collect::<Option<Vec<_>>>()
                .ok_or(SegmentMembershipBlockDenial::InvalidReference)?;
            Self::branch(tree_identity, generation, block, level, children, capacity)
        };
        decoded
            .ok_or(SegmentMembershipBlockDenial::CanonicalOrder)
            .map_err(Into::into)
    }
}
