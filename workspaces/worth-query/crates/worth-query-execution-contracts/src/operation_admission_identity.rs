//! Process-local installed operation admission identity.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static NEXT_OPERATION_ADMISSION_IDENTITY: AtomicU64 = AtomicU64::new(1);
const RESOURCE_LABEL: &str = "worth-query-application-admission:";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryOperationAdmissionIdentity(u64);

impl WorthQueryOperationAdmissionIdentity {
    pub fn prime_plateau_for_test() {
        let _ = NEXT_OPERATION_ADMISSION_IDENTITY.fetch_max(100_000, Ordering::Relaxed);
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn mint() -> Option<Self> {
        Self::mint_from(&NEXT_OPERATION_ADMISSION_IDENTITY)
    }

    pub fn resource_binding_identity(self) -> Arc<str> {
        Arc::from(format!("{RESOURCE_LABEL}{}", self.0))
    }

    pub fn mint_from(counter: &AtomicU64) -> Option<Self> {
        let mut current = counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = current.checked_add(1) else {
                break Err(current);
            };
            match counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
        .ok()
        .map(Self)
    }
}
