//! The Publication owner's live registration view for Mutation.
use super::{RelationalRuntimePublicationBinding, RelationalRuntimePublicationLifecycle};
use crate::mvcc::publication::companion::registration_stop::PublicationCompanionRegistrationStop;
use crate::validation::candidate_source::registration_observation::ManagedSourceRegistration;
use std::sync::Arc;

impl ManagedSourceRegistration for RelationalRuntimePublicationLifecycle {
    fn active_generation(&self) -> Result<Option<u64>, PublicationCompanionRegistrationStop> {
        let epoch = self.companion.enter()?;
        Ok(epoch.active()?.map(|(generation, _)| generation))
    }
}

impl RelationalRuntimePublicationBinding {
    pub(crate) fn source_registration(&self) -> Arc<dyn ManagedSourceRegistration> {
        self.lifecycle.clone()
    }
}
