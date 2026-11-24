//! One selection policy for cold inventories and ordinary generation retirement.

use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;
use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexGenerationId;
use worth_relational_contracts::internal_port::indexes::data::publication_posture::DerivedIndexPublicationStatus;

use super::GenerationCatalog;

impl GenerationCatalog {
    pub(super) fn selected_for_retention(
        &self,
        id: DerivedIndexGenerationId,
        generation: &DerivedIndexGeneration,
        global: bool,
        basis_held: bool,
        source_is_live: bool,
    ) -> bool {
        let version = generation.applicability.version_id;
        let schema = generation.applicability.schema_version;
        let branch = (!global).then_some(&generation.applicability.branch_id);
        let scope = self.scope(generation.index_id, branch);
        let exact = basis_held
            && (scope.and_then(|scope| scope.exact(version, schema)) == Some(id)
                || scope.and_then(|scope| {
                    scope.published_for_commit(generation.source_commit_id, version)
                }) == Some(id)
                || (generation.status == DerivedIndexPublicationStatus::BuildFailed
                    && scope.and_then(|scope| scope.candidate(false, version, schema))
                        == Some(id)));
        // Native publication and every artifact readmission enforce source ==
        // applicability branch, so this existing scope is the source locator.
        exact
            || source_is_live
                && self
                    .scope(generation.index_id, Some(&generation.source_branch_id))
                    .and_then(|scope| scope.latest())
                    == Some(id)
    }
}
