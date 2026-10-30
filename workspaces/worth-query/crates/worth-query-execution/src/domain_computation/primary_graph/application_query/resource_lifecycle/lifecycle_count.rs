use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) fn acquire(
    counter: &AtomicUsize,
    amount: usize,
) -> Result<usize, LifecycleCountOverflow> {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let Some(updated_value) = current.checked_add(amount) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .map(|previous| previous + amount)
    .map_err(|_| LifecycleCountOverflow)
}

pub(super) fn release(
    counter: &AtomicUsize,
    amount: usize,
) -> Result<usize, LifecycleCountUnderflow> {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let Some(updated_value) = current.checked_sub(amount) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .map(|previous| previous - amount)
    .map_err(|_| LifecycleCountUnderflow)
}

pub(super) fn record_one_saturating(counter: &AtomicUsize) {
    let _ = {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let Some(updated_value) = Some(current.saturating_add(1)) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LifecycleCountOverflow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LifecycleCountUnderflow;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_overflow_and_underflow_leave_authoritative_count_unchanged() {
        let full = AtomicUsize::new(usize::MAX);
        assert_eq!(acquire(&full, 1), Err(LifecycleCountOverflow));
        assert_eq!(full.load(Ordering::Acquire), usize::MAX);

        let empty = AtomicUsize::new(0);
        assert_eq!(release(&empty, 1), Err(LifecycleCountUnderflow));
        assert_eq!(empty.load(Ordering::Acquire), 0);
    }

    #[test]
    fn lifetime_diagnostic_count_saturates_instead_of_wrapping() {
        let count = AtomicUsize::new(usize::MAX);
        record_one_saturating(&count);
        assert_eq!(count.load(Ordering::Acquire), usize::MAX);
    }
}
