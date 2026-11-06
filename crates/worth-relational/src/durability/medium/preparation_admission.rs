//! Read-only observation of an already unresolved durable publication.
//! The one-assignment fate is independent of the lock held during medium I/O.
use super::{RelationalDurabilityUnresolved, RelationalDurableMediumBinding};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub(crate) struct RelationalDurablePreparationAdmission {
    unresolved: Option<Arc<OnceLock<RelationalDurabilityUnresolved>>>,
    home: Option<std::sync::Weak<RelationalDurableMediumBinding>>,
}

impl RelationalDurablePreparationAdmission {
    pub(crate) fn has_attached_medium(&self) -> bool {
        self.unresolved.is_some()
    }

    pub(crate) fn new(medium: Option<Arc<RelationalDurableMediumBinding>>) -> Self {
        Self {
            home: medium.as_ref().map(Arc::downgrade),
            unresolved: medium.map(|medium| medium.preparation_fate()),
        }
    }

    pub(crate) fn requires_home_program_contracts(
        &self,
    ) -> Result<bool, super::RelationalHomeProgramWriteDenial> {
        match &self.home {
            None => Ok(false),
            Some(home) => home
                .upgrade()
                .ok_or(super::RelationalHomeProgramWriteDenial::MediumMoved)?
                .requires_home_program_contracts(),
        }
    }

    pub(crate) fn admit_home_program_write(
        &self,
        step: Option<&crate::runtime::AdmittedRelationalHomeProgramStep>,
        codec: Option<&crate::durability::RecoveredCodecPublicationCustody>,
    ) -> Result<(), super::RelationalHomeProgramWriteDenial> {
        match &self.home {
            None => Ok(()),
            Some(home) => home
                .upgrade()
                .ok_or(super::RelationalHomeProgramWriteDenial::MediumMoved)?
                .admit_home_program_write(step, codec),
        }
    }

    pub(crate) fn require_open(&self) -> Result<(), RelationalDurabilityUnresolved> {
        match self.unresolved.as_ref().and_then(|fate| fate.get()) {
            Some(fate) => Err(fate.clone()),
            None => Ok(()),
        }
    }
}
