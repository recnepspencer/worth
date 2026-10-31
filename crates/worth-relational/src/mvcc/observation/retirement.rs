//! Cooperative retirement of this observation's share, never another reader's.
use super::*;

#[derive(Debug)]
pub(crate) struct RelationalObservationRetirement {
    shared: Option<Arc<AdmittedRelationalBranchBasisInner>>,
    exclusive: Option<AdmittedRelationalBranchBasisInner>,
    retention_released: bool,
}

impl RelationalBranchObservation {
    pub(crate) fn into_retirement(self) -> RelationalObservationRetirement {
        RelationalObservationRetirement {
            shared: Some(self.inner),
            exclusive: None,
            retention_released: false,
        }
    }
}

impl RelationalObservationRetirement {
    /// The caller borrows the authentic owner throughout this step. Its live
    /// head/retired-root inventory prevents provider custody destruction here.
    pub(crate) fn advance(&mut self) -> Result<bool, ()> {
        if let Some(shared) = self.shared.take() {
            self.exclusive = Arc::into_inner(shared);
            return Ok(self.exclusive.is_none());
        }
        let Some(inner) = self.exclusive.as_mut() else {
            return Ok(true);
        };
        if !self.retention_released {
            if !inner.retention.try_retire() {
                return Err(());
            }
            self.retention_released = true;
            return Ok(false);
        }
        if let Some(lease) = inner.registry_lease.get_mut() {
            if !lease.try_retire() {
                return Err(());
            }
        }
        // Both business obligations are disarmed before ordinary field Drop.
        self.exclusive.take();
        Ok(true)
    }
}
