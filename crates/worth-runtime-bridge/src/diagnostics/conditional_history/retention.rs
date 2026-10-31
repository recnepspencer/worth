//! The existing diagnostics owner retains a bounded FIFO, with no lookup registry.
use super::*;
use worth_execution::{ExecutionRequest, LeaseDenial, WorkCeilingDenial};

#[derive(Default)]
pub(crate) struct ConditionalExecutionHistory {
    head: Option<Arc<HistorySlot>>,
    tail: Option<Arc<HistorySlot>>,
    len: usize,
    configured: Option<(bool, usize)>,
    captured: bool,
}
impl ConditionalExecutionHistory {
    pub(in crate::diagnostics) fn configure_for_test(
        &mut self,
        enabled: bool,
        limit: usize,
    ) -> bool {
        if self.captured || self.configured.is_some() || limit == 0 {
            return false;
        }
        self.configured = Some((enabled, limit));
        true
    }
    pub(crate) fn capture(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        decision: &SignalConditionalDecisionEvidence,
        enabled: bool,
        limit: usize,
    ) -> Result<ConditionalHistoryReference, WorkCeilingDenial> {
        let (enabled, limit) = self.configured.unwrap_or((enabled, limit.max(1)));
        if !enabled {
            return Ok(ConditionalHistoryReference::Unretained(
                BridgeConditionalHistoryAvailability::OmittedByTier,
            ));
        }
        let memory = match execution.reserve_retained_memory(slot_bytes()) {
            Ok(memory) => memory,
            Err(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                cause @ MemoryLimitDenial::Capacity { .. },
            ))) => {
                return Ok(ConditionalHistoryReference::Unretained(
                    BridgeConditionalHistoryAvailability::CaptureDenied(cause),
                ));
            }
            Err(cause) => return Err(cause),
        };
        let slot = Arc::new(HistorySlot {
            node: Mutex::new(HistoryNode {
                state: HistoryState::Retained(BridgeConditionalHistoryDetail::from_decision(
                    decision,
                )),
                next: None,
            }),
            _memory: memory,
        });
        if let Some(tail) = &self.tail {
            tail.node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .next = Some(Arc::clone(&slot));
        } else {
            self.head = Some(Arc::clone(&slot));
        }
        self.tail = Some(Arc::clone(&slot));
        self.len += 1;
        self.captured = true;
        while self.len > limit {
            self.pop(BridgeConditionalHistoryAvailability::EvictedByRetention);
        }
        Ok(ConditionalHistoryReference::Retained(slot))
    }
    fn pop(&mut self, cause: BridgeConditionalHistoryAvailability) {
        let Some(head) = self.head.take() else {
            return;
        };
        let mut node = head
            .node
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.head = node.next.take();
        node.state = HistoryState::Lost(cause);
        self.len -= 1;
        if self.head.is_none() {
            self.tail = None;
        }
    }
}
impl Drop for ConditionalExecutionHistory {
    fn drop(&mut self) {
        while self.head.is_some() {
            self.pop(BridgeConditionalHistoryAvailability::OwnerClosed);
        }
    }
}
fn slot_bytes() -> u64 {
    let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<HistorySlot>())
        .expect("fixed history Arc layout")
        .0
        .pad_to_align();
    u64::try_from(layout.size()).expect("fixed history size fits u64")
}
