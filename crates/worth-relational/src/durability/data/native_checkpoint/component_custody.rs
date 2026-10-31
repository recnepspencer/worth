//! Shared actual capture custody survives clones and interrupted acknowledgement.
use crate::branch::AdmittedRelationalBranchBasis;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub(super) struct ComponentCaptureCustody(Option<Arc<Mutex<Vec<AdmittedRelationalBranchBasis>>>>);
impl ComponentCaptureCustody {
    pub(super) fn new(holds: Vec<AdmittedRelationalBranchBasis>) -> Self {
        if holds.is_empty() {
            Self::default()
        } else {
            Self(Some(Arc::new(Mutex::new(holds))))
        }
    }
    pub(super) fn release(&self) {
        if let Some(holds) = &self.0 {
            let retired = std::mem::take(
                &mut *holds
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            );
            // Destroy actual observation/root custody outside its shared lock.
            drop(retired);
        }
    }
}

#[cfg(test)]
mod tests;
