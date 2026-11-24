//! The Publication owner's live registration view for Mutation.
use super::{RelationalRuntimePublicationBinding, RelationalRuntimePublicationLifecycle};
use worth_relational_contracts::internal_port::mvcc::publication::companion::registration_stop::PublicationCompanionRegistrationStop;
use worth_relational_mutation::internal_port::validation::candidate_source::registration_observation::ManagedSourceRegistration;
use std::sync::Arc;

impl ManagedSourceRegistration for RelationalRuntimePublicationLifecycle {
    fn active_generation(&self) -> Result<Option<u64>, PublicationCompanionRegistrationStop> {
        let epoch = self.companion.enter()?;
        Ok(epoch.active()?.map(|(generation, _)| generation))
    }
}

impl RelationalRuntimePublicationBinding {
    pub fn source_registration(&self) -> Arc<dyn ManagedSourceRegistration> {
        self.lifecycle.clone()
    }
}
