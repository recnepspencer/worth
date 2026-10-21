//! The embedded query is complete canonical meaning, not only its display name.
use super::WorthQueryPortableApplicationContractReconstructionWork;
use worth_query_declaration::facade::application_query::ErasedApplicationQueryDefinition;

impl WorthQueryPortableApplicationContractReconstructionWork {
    pub(super) fn observe_query_context(
        &mut self,
        binding: &str,
        query: &ErasedApplicationQueryDefinition,
    ) {
        self.observe_text(binding);
        let basis = query.canonical_basis().basis().payload();
        self.observe_text(&worth_foundational::facade::canonical_basis_sequence_material(basis));
        self.nested_entries = self
            .nested_entries
            .saturating_add(u64::try_from(basis.entries().len()).unwrap_or(u64::MAX));
    }
}
