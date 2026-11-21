use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_APPLICATION_OUTCOME_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryApplicationOutcomeIdentity(u64);

impl WorthQueryApplicationOutcomeIdentity {
    pub fn mint() -> Option<Self> {
        {
            let update_counter = &NEXT_APPLICATION_OUTCOME_IDENTITY;
            let mut current = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = current.checked_add(1) else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .ok()
        .map(Self)
    }

    pub fn restore(value: u64) -> Option<Self> {
        let next = value.checked_add(1)?;
        if value == 0 {
            return None;
        }
        NEXT_APPLICATION_OUTCOME_IDENTITY.fetch_max(next, Ordering::Relaxed);
        Some(Self(value))
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}
