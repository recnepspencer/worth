//! Canonical payload projection; framing and integrity admission belong to callers.
use super::*;

impl PhysicalFreeSpaceMembershipBlock {
    /// Projects this family's payload without inspecting or authenticating a durable frame.
    /// This pure format operation grants no persisted-source or decoder authority.
    pub fn project_payload(
        payload: &[u8],
        block_identity: u64,
        capacity: u16,
        limits: FreeSpaceMembershipBlockDecodeLimits,
    ) -> Result<Self, BoundedFreeSpaceMembershipBlockDecodeDenial> {
        if payload.len() < BLOCK_PREFIX_BYTES
            || payload[21..24] != [0; 3]
            || payload[32..40] != [0; 8]
        {
            return Err(FreeSpaceRoutingDenial::Malformed.into());
        }
        let tree_identity = read_u64(payload, 0);
        let block = read_u64(payload, 8);
        let level = u16::from_le_bytes(payload[16..18].try_into().unwrap());
        let count = u16::from_le_bytes(payload[18..20].try_into().unwrap());
        let generation = read_u64(payload, 24);
        if tree_identity == 0
            || generation == 0
            || block != block_identity
            || count == 0
            || count > capacity
        {
            return Err(FreeSpaceRoutingDenial::IdentityOrCapacity.into());
        }
        let width = match payload[20] {
            1 if level == 0 => ENTRY_BYTES,
            2 if level != 0 => REFERENCE_BYTES,
            _ => return Err(FreeSpaceRoutingDenial::Malformed.into()),
        };
        if payload.len() != BLOCK_PREFIX_BYTES + usize::from(count) * width {
            return Err(FreeSpaceRoutingDenial::Malformed.into());
        }
        let observed = u64::from(count);
        if level == 0 && observed > limits.leaf_entries {
            return Err(BoundedFreeSpaceMembershipBlockDecodeDenial::LeafEntries {
                observed,
                admitted: limits.leaf_entries,
            });
        }
        if level != 0 && observed > limits.branch_children {
            return Err(
                BoundedFreeSpaceMembershipBlockDecodeDenial::BranchChildren {
                    observed,
                    admitted: limits.branch_children,
                },
            );
        }
        let body = &payload[BLOCK_PREFIX_BYTES..];
        let decoded = if level == 0 {
            Self::leaf(
                tree_identity,
                generation,
                block,
                body.chunks_exact(width)
                    .map(decode_entry)
                    .collect::<Option<Vec<_>>>()
                    .ok_or(FreeSpaceRoutingDenial::Malformed)?,
                capacity,
            )
        } else {
            Self::branch(
                tree_identity,
                generation,
                block,
                level,
                body.chunks_exact(width)
                    .map(decode_reference)
                    .collect::<Option<Vec<_>>>()
                    .ok_or(FreeSpaceRoutingDenial::InvalidReference)?,
                capacity,
            )
        };
        decoded.ok_or(FreeSpaceRoutingDenial::CanonicalOrder.into())
    }
}
