use super::*;

/// A live aggregate memory claim. Cloning this claim is deliberately unavailable;
/// shared source custody keeps its original claim through its existing Arc.
pub struct WorthQueryApplicationQueryBatchMemory {
    totals: Arc<Mutex<BatchTotals>>,
    bytes: usize,
    policy: BatchPolicy,
}

impl WorthQueryApplicationQueryBatchAdmission {
    /// Claim framework-owned storage before allocating or copying it. The
    /// logical byte quote excludes allocator overhead, as scalar buffers do.
    pub fn claim_memory(
        &self,
        bytes: usize,
    ) -> Result<WorthQueryApplicationQueryBatchMemory, WorthQueryApplicationQueryBatchResourceDenial>
    {
        let mut totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let required = totals
            .bytes
            .checked_add(bytes)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        if required > self.policy.maximum_bytes() {
            let denial = WorthQueryApplicationQueryBatchResourceDenial::MemoryLimit {
                required,
                maximum: self.policy.maximum_bytes(),
            };
            totals.memory_denial = Some(denial);
            return Err(denial);
        }
        totals.bytes = required;
        totals.peak_bytes = totals.peak_bytes.max(required);
        Ok(WorthQueryApplicationQueryBatchMemory {
            totals: Arc::clone(&self.totals),
            bytes,
            policy: self.policy.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retained_meter(
        &self,
    ) -> Self {
        Self {
            policy: self.policy.clone(),
            totals: Arc::clone(&self.totals),
        }
    }
}

impl WorthQueryApplicationQueryBatchMemory {
    pub(in crate::domain_computation::primary_graph::application_query) fn belongs_to(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> bool {
        Arc::ptr_eq(&self.totals, &batch.totals)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn grow(
        &mut self,
        bytes: usize,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let mut totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let required = totals
            .bytes
            .checked_add(bytes)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        let owned = self
            .bytes
            .checked_add(bytes)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        if required > self.policy.maximum_bytes() {
            let denial = WorthQueryApplicationQueryBatchResourceDenial::MemoryLimit {
                required,
                maximum: self.policy.maximum_bytes(),
            };
            totals.memory_denial = Some(denial);
            return Err(denial);
        }
        totals.bytes = required;
        totals.peak_bytes = totals.peak_bytes.max(required);
        self.bytes = owned;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn release(
        &mut self,
        bytes: usize,
    ) {
        assert!(
            bytes <= self.bytes,
            "a batch claim cannot release another claim's bytes"
        );
        let mut totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        totals.bytes = totals
            .bytes
            .checked_sub(bytes)
            .expect("batch memory cannot underflow");
        self.bytes -= bytes;
    }
}

impl Drop for WorthQueryApplicationQueryBatchMemory {
    fn drop(&mut self) {
        self.release(self.bytes);
    }
}
