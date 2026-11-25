//! Bind exact installed inputs to artifact assembly for the selected candidate.
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedIndexCatalogOwnerPort as _;
use super::RelationalPreparationRuntime;
use crate::authority::commit::phases::publication::CanonicalEnvelopeInputs;
use crate::authority::commit::pipeline::artifact_execution::{
    authored_index_writes::AuthoredIndexWriteInputs,
    index_catalog::PublicationIndexCatalogInputs,
    preparation::{ArtifactAuthority, PublicationArtifactOwners},
};
use crate::authority::commit::pipeline::candidate_preparation::{
    CandidateArtifactSelection, SelectedCandidateSymbols,
};
use crate::history::data::{CanonicalAuthoredIndexWrites, CanonicalIndexDefinitionDelta};

impl RelationalPreparationRuntime {
    pub(crate) fn bind_candidate_artifacts<'a>(
        &'a self,
        symbols: &'a SelectedCandidateSymbols,
        original: Option<&'a (CanonicalIndexDefinitionDelta, CanonicalAuthoredIndexWrites)>,
    ) -> CandidateArtifactSelection<'a> {
        let work = self.commit_work_budget.as_ref();
        let owners = PublicationArtifactOwners {
            index_catalog: PublicationIndexCatalogInputs {
                history: &self.history,
                checkpoint: &self.checkpoint_selection,
                catalog: &self.index_definitions,
                home_catalog: self
                    .home_target
                    .as_ref()
                    .and_then(|target| target.index_definitions.as_ref())
                    .map(|prepared| prepared.captured_catalog()),
                original_catalog: original.map(|evidence| &evidence.0),
                work,
            },
            authored_indexes: AuthoredIndexWriteInputs {
                schema_authority_digest: self.schema_contract_runtime.schema_authority_digest,
                original: original.map(|evidence| &evidence.1),
                work,
            },
            authority: ArtifactAuthority {
                publication: self.publication_preparation_authority(),
                lineage: self.lineage_preparation_authority(),
                envelope: CanonicalEnvelopeInputs {
                    history: &self.history,
                    diagnostics: self.preparation_diagnostics(),
                    symbols: &symbols.dictionary,
                    symbol_references: &symbols.references,
                },
            },
            work,
        };
        CandidateArtifactSelection {
            owners,
            max_patch_records: self.config.publication.policy.max_patch_records_per_commit,
        }
    }
}
