use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GRAPH_WORK_SESSION_IDENTITY: AtomicU64 = AtomicU64::new(1);
static NEXT_GRAPH_WORK_MANAGED_RUN_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// Fixed-width identity of the managed run that owns one provider session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryGraphWorkManagedRunIdentity(u64);

impl WorthQueryGraphWorkManagedRunIdentity {
    pub(super) fn mint() -> Option<Self> {
        {
            let update_counter = &NEXT_GRAPH_WORK_MANAGED_RUN_IDENTITY;
            let mut value = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = value.checked_add(1) else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .ok()
        .map(Self)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Fixed-width identity of one live graph-work session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryGraphWorkSessionIdentity(u64);

impl WorthQueryGraphWorkSessionIdentity {
    pub(super) fn mint() -> Option<Self> {
        {
            let update_counter = &NEXT_GRAPH_WORK_SESSION_IDENTITY;
            let mut value = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = value.checked_add(1) else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .ok()
        .map(Self)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity};

    #[test]
    fn session_identity_is_fixed_width_and_needs_no_digest_text() {
        let identity = WorthQueryGraphWorkSessionIdentity::mint().unwrap();
        assert_eq!(std::mem::size_of_val(&identity), std::mem::size_of::<u64>());
        assert!(identity.as_u64() > 0);
    }

    #[test]
    fn managed_run_identity_is_distinct_and_fixed_width() {
        let run = WorthQueryGraphWorkManagedRunIdentity::mint().unwrap();
        let session = WorthQueryGraphWorkSessionIdentity::mint().unwrap();

        assert_eq!(std::mem::size_of_val(&run), std::mem::size_of::<u64>());
        assert!(run.as_u64() > 0);
        assert!(session.as_u64() > 0);
    }
}
