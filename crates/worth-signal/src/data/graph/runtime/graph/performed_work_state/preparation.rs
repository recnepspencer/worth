use super::*;
use buffer::CapturedPerformedWork;

/// A pending capture slot, with no mutex retained across provider execution.
pub(crate) struct PreparedPerformedWorkCapture {
    buffer: Arc<Mutex<PerformedWorkBuffer>>,
    epoch: u64,
    record: Option<CapturedPerformedWork>,
    pending: bool,
}
impl PerformedWorkCaptureState {
    pub(crate) fn prepare(
        &self,
        binding: &InvalidationWorkBindingAxes,
        ledger: Option<&Arc<Ledger>>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Option<PreparedPerformedWorkCapture>, SignalError> {
        if !self
            .capture_gate
            .captures(SignalObservationSurface::PerformedWork)
        {
            return Ok(None);
        }
        let payload = match &binding.origin {
            crate::data::proof::invalidation::progression::InvalidationOriginBinding::DependencyCommit { producer_commit_ordinals, .. } => Charge::capacity::<crate::data::proof::invalidation::binding::OutputCommitOrdinal>(producer_commit_ordinals.capacity()),
            _ => Ok(Charge::ZERO),
        }.and_then(|payload| payload.checked_add(Charge::capacity::<PreparedPerformedWorkCapture>(1)?)).map_err(super::super::map_node_edit_accounting)?;
        work.reserve(
            usize::try_from(payload.bytes())
                .ok()
                .and_then(|bytes| bytes.checked_add(1)),
        )?;
        let custody = ledger
            .map(|ledger| ledger.reserve(0, payload))
            .transpose()
            .map_err(super::super::map_node_edit_retention)?;
        let epoch = self
            .bindings
            .lock()
            .expect("performed work observation poisoned")
            .reserve_pending(ledger, work)?;
        let mut prepared = PreparedPerformedWorkCapture {
            buffer: Arc::clone(&self.bindings),
            epoch,
            record: None,
            pending: true,
        };
        prepared.record = Some(CapturedPerformedWork {
            binding: binding.clone(),
            _custody: custody,
        });
        Ok(Some(prepared))
    }
}
impl PreparedPerformedWorkCapture {
    pub(crate) fn commit(mut self) {
        let mut buffer = self
            .buffer
            .lock()
            .expect("performed work observation poisoned");
        if buffer.epoch == self.epoch {
            let index = buffer.len;
            assert!(
                index < buffer.entries.len(),
                "prepared record owns capacity"
            );
            buffer.entries[index] = self.record.take();
            buffer.len += 1;
        }
        buffer.pending -= 1;
        self.pending = false;
    }
}
impl Drop for PreparedPerformedWorkCapture {
    fn drop(&mut self) {
        if self.pending {
            let mut buffer = self
                .buffer
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            buffer.pending = buffer
                .pending
                .checked_sub(1)
                .expect("capture owns pending slot");
        }
    }
}
