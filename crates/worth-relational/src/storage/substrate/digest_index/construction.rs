//! Reconstruction allocates each immutable node under its actual original grant.
use super::*;
pub(super) fn poll(
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<(), DigestIndexError> {
    if let Some(request) = request {
        request
            .consult()
            .map_err(|cause| DigestIndexError::Request(cause.into()))?;
    }
    Ok(())
}
pub(super) fn build_sorted(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    entries: &[(u128, [u8; 32])],
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<Arc<Node>, DigestIndexError> {
    if let [(key, digest)] = entries {
        return Node {
            key: *key,
            bit: 128,
            digest: OnceLock::from(hash(1, &[&key.to_be_bytes(), digest])),
            children: None,
            nodes: 1,
            _memory: None,
        }
        .allocate(request);
    }
    let bit = (entries[0].0 ^ entries[entries.len() - 1].0).leading_zeros();
    let split = entries.partition_point(|(key, _)| direction(*key, bit) == 0);
    let node = Node::branch(
        bit,
        [
            build_sorted(lane, &entries[..split], request)?,
            build_sorted(lane, &entries[split..], request)?,
        ],
    )
    .allocate(request)?;
    node.digest();
    Ok(node)
}

impl Node {
    pub(super) fn allocate(
        mut self,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<Arc<Self>, DigestIndexError> {
        if let Some(request) = request {
            self._memory = request
                .reserve_retained_memory(Arc::<Self>::layout_bytes())
                .map_err(|cause| DigestIndexError::Request(cause.into()))?;
        }
        Ok(Arc::new(self))
    }
}
