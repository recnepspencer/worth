use crate::data::error::SignalError;

/// Framework preparation consumes one explicit request scratch reservation.
/// Claims precede growth and cloning; they cannot widen the lease allowance.
pub(crate) struct SignalPreparationBudget {
    capacity: u64,
    claimed: u64,
    retained: u64,
    memory: worth_execution::ExecutionMemoryReservation,
}

/// An epoch boundary owned by the request coordinator. Only buffers allocated
/// after this mark and consumed before release may be discharged.
pub(crate) struct PreparationMark {
    transient: u64,
}

impl SignalPreparationBudget {
    pub(crate) fn for_request(
        capacity: u64,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<Self, SignalError> {
        let memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, 0)
            .map_err(SignalError::execution_admission_denied)?;
        Ok(Self {
            capacity,
            claimed: 0,
            retained: 0,
            memory,
        })
    }

    #[cfg(test)]
    pub(crate) fn new(capacity: u64) -> Self {
        let policy = worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, capacity, 10_000),
        );
        Self {
            capacity,
            claimed: 0,
            retained: 0,
            memory: worth_execution::SerialMemoryBudget::from_policy(&policy)
                .reserve(0)
                .unwrap(),
        }
    }

    pub(crate) fn remaining(&self) -> u64 {
        self.capacity - self.claimed
    }

    pub(crate) fn claim(&mut self, bytes: u64) -> Result<(), SignalError> {
        let required = self.claimed.checked_add(bytes);
        let next = required.filter(|next| *next <= self.capacity).ok_or(
            SignalError::PreparationMemoryExhausted {
                required,
                reserved: self.capacity,
            },
        )?;
        self.memory.resize(next).map_err(|denial| {
            SignalError::execution_admission_denied(worth_execution::LeaseDenial::MemoryExhausted(
                denial,
            ))
        })?;
        self.claimed = next;
        Ok(())
    }

    pub(crate) fn claim_vec<T>(&mut self, capacity: usize) -> Result<(), SignalError> {
        let bytes = capacity
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(SignalError::PreparationMemoryExhausted {
                required: None,
                reserved: self.capacity,
            })?;
        self.claim(bytes)
    }

    pub(crate) fn claim_retained(&mut self, bytes: u64) -> Result<(), SignalError> {
        self.claim(bytes)?;
        self.retained += bytes;
        Ok(())
    }

    pub(crate) fn claim_retained_vec<T>(&mut self, capacity: usize) -> Result<(), SignalError> {
        let bytes = capacity
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(SignalError::PreparationMemoryExhausted {
                required: None,
                reserved: self.capacity,
            })?;
        self.claim_retained(bytes)
    }

    pub(crate) fn checkpoint(&self) -> PreparationMark {
        PreparationMark {
            transient: self.claimed - self.retained,
        }
    }

    pub(crate) fn release(&mut self, mark: PreparationMark) {
        assert!(
            mark.transient <= self.claimed - self.retained,
            "Signal preparation frames must be released in owner order"
        );
        self.claimed = self.retained + mark.transient;
        self.memory
            .resize(self.claimed)
            .expect("releasing preparation cannot widen its reservation");
    }
}

pub(crate) fn claim_vec<T>(
    budget: Option<&mut SignalPreparationBudget>,
    capacity: usize,
) -> Result<(), SignalError> {
    if let Some(budget) = budget {
        budget.claim_vec::<T>(capacity)?;
    }
    Ok(())
}

/// Preserve Vec's ordinary geometric growth while checking its new backing
/// allocation before reserve. Each replacement remains conservatively charged
/// for the duration of this request.
pub(crate) fn push<T>(
    values: &mut Vec<T>,
    value: T,
    budget: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    if values.len() == values.capacity() {
        let capacity = values
            .capacity()
            .saturating_mul(2)
            .max(4)
            .max(values.len().saturating_add(1));
        claim_vec::<T>(budget, capacity)?;
        values.reserve_exact(capacity.saturating_sub(values.len()));
    }
    values.push(value);
    Ok(())
}
