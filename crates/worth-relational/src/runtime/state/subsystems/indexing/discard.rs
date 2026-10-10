use crate::indexes::data::{
    DerivedIndexDiscardDenial, DerivedIndexDiscardOutcome, DerivedIndexDiscardRequest,
};

use super::IndexingSubsystem;

impl IndexingSubsystem {
    pub(crate) fn discard_generations(
        &self,
        request: DerivedIndexDiscardRequest,
    ) -> Result<DerivedIndexDiscardOutcome, DerivedIndexDiscardDenial> {
        let mut state = self.state.write();
        let index_id = request.index_id();
        if !state.definitions.contains_key(&index_id) {
            return Err(DerivedIndexDiscardDenial::IndexNotInstalled { index_id });
        }
        let removed = state.generations.discard_index(index_id);
        Ok(DerivedIndexDiscardOutcome::new(index_id, removed))
    }
}
