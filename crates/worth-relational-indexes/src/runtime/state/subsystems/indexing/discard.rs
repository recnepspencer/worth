use crate::contracts::indexes::data::discard::{
    DerivedIndexDiscardDenial, DerivedIndexDiscardOutcome,
    DerivedIndexDiscardOutcomeOwnerPort as _, DerivedIndexDiscardRequest,
};

use super::IndexingSubsystem;

impl IndexingSubsystem {
    pub fn discard_generations(
        &self,
        request: DerivedIndexDiscardRequest,
    ) -> Result<DerivedIndexDiscardOutcome, DerivedIndexDiscardDenial> {
        let mut state = self.state.write();
        let index_id = request.index_id();
        if !state.catalog.definitions.contains_key(&index_id) {
            return Err(DerivedIndexDiscardDenial::IndexNotInstalled { index_id });
        }
        let removed = state.discard_catalog_generations(index_id);
        Ok(DerivedIndexDiscardOutcome::new(index_id, removed))
    }
}
