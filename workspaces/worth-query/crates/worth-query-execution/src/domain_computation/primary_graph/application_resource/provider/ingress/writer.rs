use super::super::WorthQueryResourceProviderAttempt;
use super::backing::{lock, FrameState, IngressBacking, CHUNK_BYTES};
use std::sync::{Arc, Weak};

/// Bounded ingress refusal before a producer frame is accepted for verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceIngressDenial {
    Closed,
    StaleAttempt,
    Full,
    FrameTooLarge,
}

/// Producer ingress retains only pre-admitted fixed backing, never a borrowed
/// application request. Every write checks this exact issued attempt under the
/// same synchronization that closes or reuses its slots.
#[derive(Clone)]
pub struct WorthQueryResourceIngressWriter {
    backing: Weak<IngressBacking>,
    attempt: WorthQueryResourceProviderAttempt,
}

impl WorthQueryResourceIngressWriter {
    pub fn write(&self, bytes: &[u8]) -> Result<(), WorthQueryResourceIngressDenial> {
        use WorthQueryResourceIngressDenial as Denial;
        let backing = self.backing.upgrade().ok_or(Denial::Closed)?;
        let mut lane = lock(&backing.lane);
        if !lane
            .attempt
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &self.attempt.authority))
        {
            return Err(Denial::StaleAttempt);
        }
        if lane.closed {
            return Err(Denial::Closed);
        }
        if bytes.len() > backing.maximum_frame_bytes {
            return Err(Denial::FrameTooLarge);
        }
        let index = lane.free.ok_or(Denial::Full)?;
        let mut frame = lock(&backing.frames[index]);
        lane.free = frame.next;
        for (offset, bytes) in bytes.chunks(CHUNK_BYTES).enumerate() {
            lock(&backing.chunks[index * backing.chunks_per_frame + offset])[..bytes.len()]
                .copy_from_slice(bytes);
        }
        frame.state = FrameState::Buffered;
        frame.length = bytes.len();
        frame.next = None;
        if let Some(tail) = lane.tail {
            lock(&backing.frames[tail]).next = Some(index);
        } else {
            lane.ready = Some(index);
        }
        lane.tail = Some(index);
        Ok(())
    }
}

impl IngressBacking {
    pub(in crate::domain_computation::primary_graph::application_resource) fn open(
        self: &Arc<Self>,
        attempt: &WorthQueryResourceProviderAttempt,
    ) -> Result<WorthQueryResourceIngressWriter, WorthQueryResourceIngressDenial> {
        let mut lane = lock(&self.lane);
        if !lane.closed || lane.ready.is_some() || lane.sealed.is_some() {
            return Err(WorthQueryResourceIngressDenial::Full);
        }
        lane.attempt = Some(Arc::clone(&attempt.authority));
        lane.closed = false;
        Ok(WorthQueryResourceIngressWriter {
            backing: Arc::downgrade(self),
            attempt: attempt.clone(),
        })
    }

    /// Releases unverified buffered frames only. Native queue drain and actual
    /// provider cleanup remain separate row-owned obligations.
    pub(in crate::domain_computation::primary_graph::application_resource) fn close(
        &self,
        attempt: &WorthQueryResourceProviderAttempt,
    ) -> Result<bool, WorthQueryResourceIngressDenial> {
        let mut lane = lock(&self.lane);
        if !lane
            .attempt
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &attempt.authority))
        {
            return Err(WorthQueryResourceIngressDenial::StaleAttempt);
        }
        lane.closed = true;
        while let Some(index) = lane.ready {
            let mut frame = lock(&self.frames[index]);
            lane.ready = frame.next;
            frame.state = FrameState::Free;
            frame.length = 0;
            frame.next = lane.free;
            lane.free = Some(index);
        }
        lane.tail = None;
        Ok(lane.sealed.is_none())
    }
}
