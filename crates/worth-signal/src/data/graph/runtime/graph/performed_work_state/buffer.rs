use super::*;

#[derive(Debug)]
pub(super) struct CapturedPerformedWork {
    pub(super) binding: InvalidationWorkBindingAxes,
    pub(super) _custody: Option<Reservation>,
}

/// Storage and outstanding reservations are independent of callback lifetime.
#[derive(Debug, Default)]
pub(crate) struct PerformedWorkBuffer {
    pub(super) entries: Box<[Option<CapturedPerformedWork>]>,
    pub(super) len: usize,
    pub(super) pending: usize,
    pub(super) epoch: u64,
    capacity_custody: Option<Reservation>,
    pub(super) storage_custody: Option<Arc<Reservation>>,
}
impl PerformedWorkBuffer {
    pub(crate) fn clear(&mut self) {
        self.epoch = self
            .epoch
            .checked_add(1)
            .expect("capture storage epoch exhausted");
        for entry in &mut self.entries[..self.len] {
            *entry = None;
        }
        self.len = 0;
    }

    pub(super) fn reserve_pending(
        &mut self,
        ledger: Option<&Arc<Ledger>>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<u64, SignalError> {
        let needed = self
            .len
            .checked_add(self.pending)
            .and_then(|n| n.checked_add(1))
            .ok_or(SignalError::EvaluationStorageCapacityExhausted)?;
        if needed > self.entries.len() {
            let capacity = self
                .entries
                .len()
                .checked_mul(2)
                .unwrap_or(needed)
                .max(needed)
                .max(8);
            work.reserve(capacity.checked_add(self.len).and_then(|n| {
                n.checked_mul(std::mem::size_of::<Option<CapturedPerformedWork>>())
            }))?;
            let charge = Charge::capacity::<Option<CapturedPerformedWork>>(capacity)
                .map_err(super::super::map_node_edit_accounting)?;
            let custody = ledger
                .map(|ledger| ledger.reserve(0, charge))
                .transpose()
                .map_err(super::super::map_node_edit_retention)?;
            let mut entries = Vec::with_capacity(capacity);
            entries.resize_with(capacity, || None);
            for (destination, source) in entries
                .iter_mut()
                .zip(self.entries.iter_mut())
                .take(self.len)
            {
                *destination = source.take();
            }
            // Destroy the old allocation before releasing its reservation.
            self.entries = entries.into_boxed_slice();
            self.capacity_custody = custody;
        }
        self.pending += 1;
        Ok(self.epoch)
    }
}
