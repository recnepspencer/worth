//! Moved whole from `domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation`: the items below and every impl they own.
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub(in crate::domain_computation) static NEXT_OPERATION_ADMISSION_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryOperationAdmissionIdentity(u64);

impl WorthQueryOperationAdmissionIdentity {
    pub(in crate::domain_computation) fn mint() -> Option<Self> {
        Self::mint_from(&NEXT_OPERATION_ADMISSION_IDENTITY)
    }

    pub(in crate::domain_computation) fn resource_binding_identity(
        self,
    ) -> Arc<str> {
        Arc::from(format!("{RESOURCE_LABEL}{}", self.0))
    }

    fn mint_from(counter: &AtomicU64) -> Option<Self> {
        {
            let update_counter = counter;
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
}
