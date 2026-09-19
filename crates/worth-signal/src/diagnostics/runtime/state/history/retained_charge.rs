use super::DiagnosticHistory;
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticHistoryEditDenial {
    PreparationRequired,
    PositionExhausted,
    Accounting(RetainedStoragePreparationDenial),
    CapacityExhausted { maximum: Charge, required: Charge },
}

impl From<RetainedStoragePreparationDenial> for DiagnosticHistoryEditDenial {
    fn from(value: RetainedStoragePreparationDenial) -> Self {
        Self::Accounting(value)
    }
}

impl<T> DiagnosticHistory<T> {
    /// A carried fact only. Ordinary mutation never repairs missing accounting.
    pub(crate) fn prepared_retained_charge(&self) -> Result<Charge, DiagnosticHistoryEditDenial> {
        self.retained_charge
            .get()
            .copied()
            .ok_or(DiagnosticHistoryEditDenial::PreparationRequired)
    }
}

impl<T: RetainedStorageMeasurement> DiagnosticHistory<T> {
    /// Explicit cold admission; existing prepared roots need no history walk.
    pub(crate) fn prepare_retained_charge(
        &self,
        work: &mut Work,
    ) -> Result<Charge, RetainedStoragePreparationDenial> {
        if let Some(charge) = self.retained_charge.get() {
            return Ok(*charge);
        }
        let charge = self.retained_heap_charge(work)?;
        // Concurrent cold preparation may measure twice but installs one fact.
        Ok(*self.retained_charge.get_or_init(|| charge))
    }

    /// Admit before installing a new frame. The caller separately owns the
    /// incoming frame and old/draft root custody; sharing is not a release.
    pub(crate) fn append_with_retained_capacity(
        &mut self,
        value: T,
        maximum: Charge,
        work: &mut Work,
    ) -> Result<(), (T, DiagnosticHistoryEditDenial)> {
        let admission = (|| {
            let position = self
                .next_position
                .ok_or(DiagnosticHistoryEditDenial::PositionExhausted)?;
            let previous = self.prepared_retained_charge()?;
            work.visit()?;
            let frame =
                arc_allocation_charge::<T>()?.checked_add(value.retained_heap_charge(work)?)?;
            let next_len = self
                .len()
                .checked_add(1)
                .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)?;
            let charge = previous
                .checked_sub(ordered_index_charge::<u64, Arc<T>>(self.len())?)?
                .checked_add(ordered_index_charge::<u64, Arc<T>>(next_len)?)?
                .checked_add(frame)?;
            if charge > maximum {
                return Err(DiagnosticHistoryEditDenial::CapacityExhausted {
                    maximum,
                    required: charge,
                });
            }
            Ok((position, charge))
        })();
        let (position, charge) = match admission {
            Ok(admitted) => admitted,
            Err(denial) => return Err((value, denial)),
        };
        let _ = self.retained_charge.take();
        self.entries.insert(position, Arc::new(value));
        self.next_position = position.checked_add(1);
        self.retained_charge = OnceLock::from(charge);
        Ok(())
    }

    /// The returned frame remains live custody. Only this root's charge falls;
    /// another retained root or the returned Arc can still own the allocation.
    pub(crate) fn evict_front_with_retained_charge(
        &mut self,
        work: &mut Work,
    ) -> Result<Option<Arc<T>>, DiagnosticHistoryEditDenial> {
        let previous = self.prepared_retained_charge()?;
        work.visit()?;
        let Some((&position, frame)) = self.entries.iter().next() else {
            return Ok(None);
        };
        let charge = previous
            .checked_sub(ordered_index_charge::<u64, Arc<T>>(self.len())?)?
            .checked_sub(frame.retained_heap_charge(work)?)?
            .checked_add(ordered_index_charge::<u64, Arc<T>>(self.len() - 1)?)?;
        let _ = self.retained_charge.take();
        let removed = self.entries.remove(&position);
        self.retained_charge = OnceLock::from(charge);
        Ok(removed)
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for DiagnosticHistory<T> {
    fn retained_heap_charge(
        &self,
        work: &mut Work,
    ) -> Result<Charge, RetainedStoragePreparationDenial> {
        self.entries.retained_heap_charge(work)
    }
}
