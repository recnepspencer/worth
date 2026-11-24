//! Read-only observation of an already unresolved durable publication.
//! The one-assignment fate is independent of the lock held during medium I/O.
use super::RelationalDurableMediumBinding;
use contracts::durability::medium::unresolved::RelationalDurabilityUnresolved;
use std::sync::{Arc, OnceLock};
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug, Clone)]
pub struct RelationalDurablePreparationAdmission {
    unresolved: Option<Arc<OnceLock<RelationalDurabilityUnresolved>>>,
    home: Option<std::sync::Weak<RelationalDurableMediumBinding>>,
}

impl RelationalDurablePreparationAdmission {
    pub fn has_attached_medium(&self) -> bool {
        self.unresolved.is_some()
    }

    pub fn new(medium: Option<Arc<RelationalDurableMediumBinding>>) -> Self {
        Self {
            home: medium.as_ref().map(Arc::downgrade),
            unresolved: medium.map(|medium| medium.preparation_fate()),
        }
    }

    pub fn requires_home_program_contracts(
        &self,
    ) -> Result<
        bool,
        contracts::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial,
    > {
        match &self.home {
            None => Ok(false),
            Some(home) => home
                .upgrade()
                .ok_or(contracts::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial::MediumMoved)?
                .requires_home_program_contracts(),
        }
    }

    pub fn admit_home_program_write(
        &self,
        step: Option<&worth_relational_branch::internal_port::branch::home_program_step::AdmittedRelationalHomeProgramStep>,
        codec: Option<&crate::durability::authority::RecoveredCodecPublicationCustody>,
    ) -> Result<
        (),
        contracts::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial,
    > {
        match &self.home {
            None => Ok(()),
            Some(home) => home
                .upgrade()
                .ok_or(contracts::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial::MediumMoved)?
                .admit_home_program_write(step, codec),
        }
    }

    pub fn require_open(&self) -> Result<(), RelationalDurabilityUnresolved> {
        match self.unresolved.as_ref().and_then(|fate| fate.get()) {
            Some(fate) => Err(fate.clone()),
            None => Ok(()),
        }
    }
}
