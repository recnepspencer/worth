//! Keep an exact prepared prefix current until its original effect returns.
use super::*;
use crate::contracts::mvcc::publication::outcome::refusals::RelationalPublicationDenial;

pub struct IndexDefinitionPublication<'owner> {
    _guard: std::sync::RwLockWriteGuard<'owner, ()>,
    binding: &'owner IndexDefinitionReadBinding,
    prepared: Arc<PreparedRelationalIndexDefinitions>,
}

impl IndexDefinitionReadBinding {
    /// Hold installed definition meaning through an ordinary publication.
    pub fn hold_definition_catalog(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, ()>, RelationalPublicationDenial> {
        self.definition_publication
            .try_read()
            .ok_or(RelationalPublicationDenial::IndexDefinitionPublicationContended)
    }

    pub fn prepare_definition_publication(
        &self,
        prepared: Arc<PreparedRelationalIndexDefinitions>,
    ) -> Result<IndexDefinitionPublication<'_>, RelationalPublicationDenial> {
        let guard = self
            .definition_publication
            .try_write()
            .ok_or(RelationalPublicationDenial::IndexDefinitionPublicationContended)?;
        if !prepared.admits(&self.capture_catalog()) {
            return Err(RelationalPublicationDenial::StaleIndexDefinitions);
        }
        Ok(IndexDefinitionPublication {
            _guard: guard,
            binding: self,
            prepared,
        })
    }
}
impl IndexDefinitionPublication<'_> {
    pub fn publish(self) {
        let mut state = self.binding.state.write();
        state.catalog = self.prepared.captured.catalog.clone();
        state.next_index_id = self.prepared.captured.next_index_id;
    }
}

impl IndexingSubsystem {
    /// Allocate the next definition identity and record the definition.
    pub fn register_definition(
        &self,
        mut definition: DerivedIndexDefinition,
    ) -> Result<
        DerivedIndexDefinition,
        crate::contracts::indexes::data::registration_denial::DerivedIndexRegistrationDenial,
    > {
        let _definitions = self
            .definition_publication
            .try_write()
            .ok_or(crate::contracts::indexes::data::registration_denial::DerivedIndexRegistrationDenial::PublicationInProgress)?;
        let mut state = self.state.write();
        definition.index_id = DerivedIndexId(state.next_index_id);
        state.next_index_id = state
            .next_index_id
            .checked_add(1)
            .ok_or(crate::contracts::indexes::data::registration_denial::DerivedIndexRegistrationDenial::IdentityExhausted)?;
        state.insert_definition(definition.clone());
        Ok(definition)
    }
}
