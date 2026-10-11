use super::*;

pub(super) struct IndexGenerationPublicationBasis {
    pub(super) source_commit_id: CommitId,
    pub(super) branch_id: crate::history::data::BranchId,
    pub(super) version_id: crate::identity::data::VersionId,
    pub(super) schema_version: crate::schema::data::SchemaVersionId,
}

impl IndexGenerationPublicationBasis {
    pub(super) fn new(
        request: &DerivedIndexBuildRequest,
        branch_id: crate::history::data::BranchId,
        version_id: crate::identity::data::VersionId,
        schema_version: crate::schema::data::SchemaVersionId,
    ) -> Self {
        Self {
            source_commit_id: request.source_commit_id,
            branch_id,
            version_id,
            schema_version,
        }
    }
}

pub(super) fn publish_index_generations(
    runtime: &RelationalRuntime,
    basis: &IndexGenerationPublicationBasis,
    results: Vec<(
        crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
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
        generations.push(publish_prepared_generation(
            runtime,
            basis,
            result.index_id,
            entries,
        ));
    }
    generations
}

pub(super) fn publish_prepared_generation(
    runtime: &RelationalRuntime,
    basis: &IndexGenerationPublicationBasis,
    index_id: DerivedIndexId,
    entries: DerivedIndexEntries,
) -> DerivedIndexGeneration {
    let generation = prepared_generation(
        basis,
        DerivedIndexGenerationId(runtime.indexes.next_generation_id()),
        index_id,
        entries,
    );
    runtime.indexes.publish_generation(generation.clone());
    generation
}

pub(super) fn prepared_generation(
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
