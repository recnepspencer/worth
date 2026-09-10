use super::DiagnosticHistory;
use crate::data::retained_storage::{
    ordered_lookup_steps, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial as Denial,
};
impl<T> DiagnosticHistory<T> {
    /// im 15.1 path search plus split/merge and COW of adjacent 64-slot nodes.
    /// Eight lookup-sized passes cover navigation and root-to-leaf structural
    /// copies; u64 keys and Arc frame handles have fixed comparison/copy cost.
    pub(crate) fn admit_edit_work(&self, work: &mut Work) -> Result<(), Denial> {
        let steps = ordered_lookup_steps(self.len().checked_add(1).ok_or(Denial::ChargeOverflow)?)
            .checked_mul(8)
            .ok_or(Denial::ChargeOverflow)?;
        work.reserve_visits(steps)
    }
}
