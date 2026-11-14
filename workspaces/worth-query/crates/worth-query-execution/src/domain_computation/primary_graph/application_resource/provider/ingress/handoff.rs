//! Freeze accepted ingress before the live close discards its backing.
use super::super::WorthQueryResourceProviderAttempt;
use super::backing::{lock, IngressBacking, CHUNK_BYTES};
use super::WorthQueryResourceIngressDenial;
use std::sync::Arc;
impl IngressBacking {
    pub(in crate::domain_computation::primary_graph::application_resource) fn freeze_for_handoff(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
    ) -> Result<Vec<Vec<u8>>, WorthQueryResourceIngressDenial> {
        let mut lane = lock(&self.lane);
        if !lane
            .attempt
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &attempt.authority))
        {
            return Err(WorthQueryResourceIngressDenial::StaleAttempt);
        }
        lane.closed = true;
        let mut frames = Vec::new();
        // A sealed candidate precedes every still-buffered frame.
        let mut indexes = Vec::new();
        if let Some(index) = lane.sealed {
            indexes.push(index);
        }
        let mut next = lane.ready;
        while let Some(index) = next {
            indexes.push(index);
            next = lock(&self.frames[index]).next;
        }
        for index in indexes {
            let length = lock(&self.frames[index]).length;
            let mut bytes = Vec::with_capacity(length);
            for offset in 0..length.div_ceil(CHUNK_BYTES) {
                let chunk = lock(&self.chunks[index * self.chunks_per_frame + offset]);
                bytes.extend_from_slice(&chunk[..(length - offset * CHUNK_BYTES).min(CHUNK_BYTES)]);
            }
            frames.push(bytes);
        }
        Ok(frames)
    }
}
