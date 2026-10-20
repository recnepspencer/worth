use super::*;

impl BatchPolicy {
    // The caller holds totals, preserving the shared totals-before-plan order.
    fn start_claim(&self) {
        if let Self::Installed(plan) = self {
            plan.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .started = true;
        }
    }
}

/// A live aggregate memory claim. Cloning this claim is deliberately unavailable;
/// shared source custody keeps its original claim through its existing Arc.
pub struct WorthQueryApplicationQueryBatchMemory {
    totals: Arc<Mutex<BatchTotals>>,
    bytes: usize,
    policy: BatchPolicy,
}

impl WorthQueryApplicationQueryBatchAdmission {
    /// Quotes actual collection/source backing before claiming or copying it.
    /// Installed schedules add the owner's exact envelope; explicit or narrowed
    /// custody ceilings still refuse excess, without changing read-work charges.
    /// This storage loan grants no query or disclosure authority.
    pub fn claim_publication_memory(
        &self,
        bytes: usize,
    ) -> Result<WorthQueryApplicationQueryBatchMemory, WorthQueryApplicationQueryBatchResourceDenial>
    {
        let totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.policy.start_claim();
        if let BatchPolicy::Installed(plan) = &self.policy {
            let mut plan = plan
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let quoted = plan
                .bytes
                .checked_add(bytes)
                .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
            plan.bytes = plan
                .byte_ceiling
                .map_or(quoted, |maximum| quoted.min(maximum));
        }
        drop(totals);
        self.claim_memory(bytes)
    }

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
        self.policy.start_claim();
        let required = totals
            .bytes
            .checked_add(bytes)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        if let Some(maximum) = self
            .policy
            .maximum_bytes()
            .filter(|maximum| required > *maximum)
        {
            let denial =
                WorthQueryApplicationQueryBatchResourceDenial::MemoryLimit { required, maximum };
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
    pub(in crate::domain_computation::primary_graph::application_query) fn grow_to(
        &mut self,
        bytes: usize,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        if let Some(extra) = bytes.checked_sub(self.bytes) {
            self.grow(extra)?;
        }
        Ok(())
    }

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
        if let Some(maximum) = self
            .policy
            .maximum_bytes()
            .filter(|maximum| required > *maximum)
        {
            let denial =
                WorthQueryApplicationQueryBatchResourceDenial::MemoryLimit { required, maximum };
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
