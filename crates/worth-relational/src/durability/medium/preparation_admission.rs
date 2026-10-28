//! Read-only observation of an already unresolved durable publication.
//! The one-assignment fate is independent of the lock held during medium I/O.
use super::{RelationalDurabilityUnresolved, RelationalDurableMediumBinding};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub(crate) struct RelationalDurablePreparationAdmission {
    unresolved: Option<Arc<OnceLock<RelationalDurabilityUnresolved>>>,
}

impl RelationalDurablePreparationAdmission {
    pub(crate) fn new(medium: Option<Arc<RelationalDurableMediumBinding>>) -> Self {
        Self {
            unresolved: medium.map(|medium| medium.preparation_fate()),
        }
    }

    pub(crate) fn require_open(&self) -> Result<(), RelationalDurabilityUnresolved> {
        match self.unresolved.as_ref().and_then(|fate| fate.get()) {
            Some(fate) => Err(fate.clone()),
            None => Ok(()),
        }
    }
}
