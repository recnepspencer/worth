use super::IndexPreparationResult;
use crate::branch::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexEntries, DerivedIndexGeneration,
};
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::indexes::data::definition::{DerivedIndexGenerationId, DerivedIndexId};
use crate::contracts::indexes::data::publication_posture::{
    DerivedIndexApplicability, DerivedIndexPublicationStatus,
};
use crate::runtime::state::subsystems::indexing::IndexingSubsystem;

pub struct IndexGenerationPublicationBasis {
    pub(super) source_commit_id: CommitId,
    pub(super) branch_id: crate::contracts::history::data::coordinates::BranchId,
    pub(super) version_id: crate::contracts::identity::data::primitives::VersionId,
    pub(super) schema_version: crate::contracts::schema::data::SchemaVersionId,
}

impl IndexGenerationPublicationBasis {
    pub(super) fn new(
        request: &DerivedIndexBuildRequest,
        branch_id: crate::contracts::history::data::coordinates::BranchId,
        version_id: crate::contracts::identity::data::primitives::VersionId,
        schema_version: crate::contracts::schema::data::SchemaVersionId,
    ) -> Self {
        Self {
            source_commit_id: request.source_commit_id,
            branch_id,
            version_id,
            schema_version,
        }
    }
}

pub fn publish_index_generations(
    indexes: &IndexingSubsystem,
    basis: &IndexGenerationPublicationBasis,
    results: Vec<(
        crate::authority::commit::preparation::packets::index::IndexReductionKey,
        IndexPreparationResult,
    )>,
    failed_indexes: &mut Vec<DerivedIndexId>,
) -> Vec<DerivedIndexGeneration> {
    let mut generations = Vec::new();
    for (_, result) in results {
        let Some(entries) = result.entries else {
            failed_indexes.push(result.index_id);
            continue;
        };
        match publish_prepared_generation(indexes, basis, result.index_id, entries) {
            Some(generation) => generations.push(generation),
            None => failed_indexes.push(result.index_id),
        }
    }
    generations
}

pub(super) fn publish_prepared_generation(
    indexes: &IndexingSubsystem,
    basis: &IndexGenerationPublicationBasis,
    index_id: DerivedIndexId,
    entries: DerivedIndexEntries,
) -> Option<DerivedIndexGeneration> {
    let generation = prepared_generation(
        basis,
        DerivedIndexGenerationId(indexes.next_generation_id()?),
        index_id,
        entries,
    );
    indexes.publish_generation(generation.clone());
    Some(generation)
}

pub fn prepared_generation(
    basis: &IndexGenerationPublicationBasis,
    generation_id: DerivedIndexGenerationId,
    index_id: DerivedIndexId,
    entries: DerivedIndexEntries,
) -> DerivedIndexGeneration {
    DerivedIndexGeneration {
        generation_id,
        index_id,
        source_commit_id: basis.source_commit_id,
        source_branch_id: basis.branch_id.clone(),
        applicability: DerivedIndexApplicability {
            branch_id: basis.branch_id.clone(),
            version_id: basis.version_id,
            schema_version: basis.schema_version,
        },
        status: DerivedIndexPublicationStatus::Published,
        entries,
    }
}
