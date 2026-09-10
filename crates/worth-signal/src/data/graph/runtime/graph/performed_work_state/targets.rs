use super::*;
use crate::data::handle::NodeId;

pub(crate) struct PerformedTargetSnapshot {
    pub(crate) targets: Vec<NodeId>,
    pub(crate) custody: Option<Reservation>,
}
impl PerformedWorkCaptureState {
    pub(crate) fn snapshot_targets(
        &self,
        ledger: Option<&Arc<Ledger>>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<PerformedTargetSnapshot, SignalError> {
        let bindings = self
            .bindings
            .lock()
            .expect("performed work observation poisoned");
        let count = bindings.len;
        // Fixed-width targets: copy, sort and deduplicate within a conservative
        // n log(n) comparison/navigation allowance; no binding payload clones.
        let levels = (usize::BITS - count.leading_zeros()).max(1) as usize;
        work.reserve(
            count
                .checked_mul(levels)
                .and_then(|n| n.checked_mul(16))
                .and_then(|n| n.checked_add(1)),
        )?;
        let charge = Charge::capacity::<NodeId>(count)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<
                    crate::data::proof::SignalInvalidationExecutionReceipt,
                >(1)?)
            })
            .map_err(super::super::map_node_edit_accounting)?;
        let custody = ledger
            .map(|ledger| ledger.reserve(0, charge))
            .transpose()
            .map_err(super::super::map_node_edit_retention)?;
        let mut targets = Vec::with_capacity(count);
        for record in &bindings.entries[..count] {
            targets.push(
                record
                    .as_ref()
                    .expect("completed capture slot")
                    .binding
                    .target,
            );
        }
        drop(bindings);
        targets.sort_unstable();
        targets.dedup();
        Ok(PerformedTargetSnapshot { targets, custody })
    }
}

impl PerformedTargetSnapshot {
    pub(crate) fn empty(
        ledger: Option<&Arc<Ledger>>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Self, SignalError> {
        work.reserve(Some(1))?;
        let charge = Charge::capacity::<crate::data::proof::SignalInvalidationExecutionReceipt>(1)
            .map_err(super::super::map_node_edit_accounting)?;
        let custody = ledger
            .map(|ledger| ledger.reserve(0, charge))
            .transpose()
            .map_err(super::super::map_node_edit_retention)?;
        Ok(Self {
            targets: Vec::new(),
            custody,
        })
    }
}
