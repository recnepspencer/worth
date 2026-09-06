use super::super::ports::RuntimeWorldRecoveryService;
use super::super::RuntimeWorldOwnerUnavailable;
use crate::lifecycle::RuntimeWorldServiceDenial;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's recovery service.
#[derive(Clone)]
pub struct RuntimeWorldRecoveryPort {
    owner: Weak<dyn RuntimeWorldRecoveryService + Send + Sync>,
}
impl RuntimeWorldRecoveryPort {
    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldRecoveryService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldRecoveryService + Send + Sync>, RuntimeWorldOwnerUnavailable>
    {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn inspect_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
    ) -> Result<
        crate::recovery::ProductUnpublishedOwnerEffects,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        self.service()?.inspect_effects(handle)
    }
    pub fn release_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
        minimum_age_ticks: u64,
    ) -> Result<Vec<crate::branch::OwnerRetirementWork>, crate::recovery::RuntimeWorldRecoveryDenial>
    {
        self.service()?.release_effects(handle, minimum_age_ticks)
    }
    pub fn continue_effects(
        &self,
        effects: crate::recovery::ProductUnpublishedOwnerEffects,
    ) -> Result<
        crate::recovery::RecoveryContinuationContract,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        self.service()?.continue_effects(effects)
    }
    pub fn recover_performed(
        &self,
        identity: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        crate::publication::PerformedCompositePublication,
        RuntimeWorldServiceDenial<crate::recovery::PerformedPublicationRecoveryDenial>,
    > {
        self.service()?
            .recover_performed(identity)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
}
