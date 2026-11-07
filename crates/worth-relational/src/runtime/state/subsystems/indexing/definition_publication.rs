//! Keep an exact prepared prefix current until its original effect returns.
use super::*;

pub(crate) struct IndexDefinitionPublication<'owner> {
    _guard: std::sync::RwLockWriteGuard<'owner, ()>,
    binding: &'owner IndexDefinitionReadBinding,
    prepared: Arc<PreparedRelationalIndexDefinitions>,
}

impl IndexDefinitionReadBinding {
    pub(crate) fn prepare_publication(
        &self,
        prepared: Arc<PreparedRelationalIndexDefinitions>,
    ) -> Result<IndexDefinitionPublication<'_>, crate::mvcc::RelationalPublicationDenial> {
        let guard = self
            .definition_publication
            .try_write()
            .ok_or(crate::mvcc::RelationalPublicationDenial::IndexDefinitionPublicationContended)?;
        if !prepared.admits(&self.capture_catalog()) {
            return Err(crate::mvcc::RelationalPublicationDenial::StaleIndexDefinitions);
        }
        Ok(IndexDefinitionPublication {
            _guard: guard,
            binding: self,
            prepared,
        })
    }
}
impl IndexDefinitionPublication<'_> {
    pub(crate) fn publish(self) {
        let mut state = self.binding.state.write();
        state.catalog = self.prepared.captured.catalog.clone();
        state.next_index_id = self.prepared.captured.next_index_id;
    }
}

impl IndexingSubsystem {
    /// Allocate the next definition identity and record the definition.
    pub(crate) fn register_definition(
        &self,
        mut definition: DerivedIndexDefinition,
    ) -> Result<DerivedIndexDefinition, crate::indexes::data::DerivedIndexRegistrationDenial> {
        let _definitions = self
            .definition_publication
            .try_read()
            .ok_or(crate::indexes::data::DerivedIndexRegistrationDenial::PublicationInProgress)?;
        let mut state = self.state.write();
        definition.index_id = DerivedIndexId(state.next_index_id);
        state.next_index_id = state
            .next_index_id
            .checked_add(1)
            .ok_or(crate::indexes::data::DerivedIndexRegistrationDenial::IdentityExhausted)?;
        state.insert_definition(definition.clone());
        Ok(definition)
    }
}
