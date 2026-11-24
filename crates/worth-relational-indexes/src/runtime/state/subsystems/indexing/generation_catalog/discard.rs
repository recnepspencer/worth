use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexId;

use super::GenerationCatalog;

impl GenerationCatalog {
    /// The global scope contains every status, branch and basis for this index.
    /// Traverse that membership alone, removing all secondary selection routes.
    pub(in crate::internal_port::runtime::state::subsystems::indexing) fn discard_index(
        &mut self,
        index_id: DerivedIndexId,
    ) -> usize {
        let mut removed = 0;
        while let Some(id) = self.scope(index_id, None).and_then(|scope| scope.latest()) {
            let generation = self.entries.remove(&id).expect(/* Proof: Selection records IDs from this same catalog under exclusive mutation; no intervening removal occurs before consuming the selected entries. */ "catalog scope has payload");
            self.remove_bindings(&generation);
            removed += 1;
        }
        removed
    }
}
