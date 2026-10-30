#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryBoundCapabilityGeneration(u64);

impl WorthQueryBoundCapabilityGeneration {
    pub(crate) fn mint() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let ordinal = {
            let update_counter = &NEXT;
            let mut current = update_counter.load(std::sync::atomic::Ordering::Relaxed);
            loop {
                let Some(updated_value) = current.checked_add(1) else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .expect("bound capability generation space exhausted");
        Self(ordinal)
    }

    pub const fn ordinal(self) -> u64 {
        self.0
    }
}
