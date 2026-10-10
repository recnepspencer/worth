use crate::indexes::data::DerivedIndexId;

use super::GenerationCatalog;

impl GenerationCatalog {
    /// The global scope contains every status, branch and basis for this index.
    /// Traverse that membership alone, removing all secondary selection routes.
    pub(in crate::runtime::state::subsystems::indexing) fn discard_index(
        &mut self,
        index_id: DerivedIndexId,
    ) -> usize {
        let mut removed = 0;
        while let Some(id) = self.scope(index_id, None).and_then(|scope| scope.latest()) {
            let generation = self.entries.remove(&id).expect("catalog scope has payload");
            self.remove_bindings(&generation);
            removed += 1;
        }
        removed
    }
}
