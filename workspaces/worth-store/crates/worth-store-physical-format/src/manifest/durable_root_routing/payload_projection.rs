//! Owned projection delegates the exact routing grammar to the borrowed view.
use super::*;

impl PhysicalRootRoutingBlock {
    /// Projects this family's payload without inspecting a durable frame.
    /// This pure format operation grants no persisted-source authority.
    pub fn project_payload(
        payload: &[u8],
        block_identity: u64,
        capacity: u16,
        limits: RootRoutingBlockDecodeLimits,
        schema: u8,
    ) -> Result<Self, BoundedRootRoutingBlockDecodeDenial> {
        let preflight = RootRoutingBlockPreflight::inspect_payload(
            payload,
            block_identity,
            capacity,
            limits,
            schema,
        )?;
        let mut scratch = Vec::with_capacity(preflight.coordinate_scratch_slots());
        Ok(preflight.validate(&mut scratch)?.into_owned())
    }
}
