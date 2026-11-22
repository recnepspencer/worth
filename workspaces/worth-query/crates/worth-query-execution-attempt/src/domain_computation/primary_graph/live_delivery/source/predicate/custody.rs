//! Unique guards release counts; retired queue/grant custody leaves the gate.
use super::*;

fn release(
    source: &Mutex<WorthQueryLiveDeliverySourceState>,
    key: &WorthQueryLiveProductKey,
    preparation: bool,
) {
    let retired = {
        let mut state = source
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(partition) = state.partitions.get_mut(key) else {
            return;
        };
        let count = if preparation {
            &mut partition.preparation_count
        } else {
            &mut partition.opening_count
        };
        *count = count
            .checked_sub(1)
            .expect("unique guard owns an actual partition count");
        partition.is_unowned().then(|| state.remove_partition(key))
    };
    drop(retired);
}
impl Drop for WorthQueryLivePredicatePreparation {
    fn drop(&mut self) {
        release(&self.state, &self.key, true);
    }
}
impl Drop for WorthQueryLivePredicateOpening {
    fn drop(&mut self) {
        release(&self.state, &self.key, false);
    }
}
