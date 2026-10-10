mod basis_build;
mod build_execution;
mod commit_build;
mod diagnostics;
mod discard;
mod maintenance;
pub(crate) use maintenance::candidate::PreparedCandidateIndexPublication;
mod packet_planning;

use crate::history::data::CommitId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexBuildOutcome, DerivedIndexBuildRequest,
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexGeneration, DerivedIndexGenerationId,
    DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::runtime::RelationalRuntime;

use self::build_execution::{
    execute_index_inputs, execute_index_packets, index_execution_denial,
    record_index_preparation_strategy_counters, IndexPreparationResult,
};
use self::diagnostics::{
    derived_index_build_artifact_kind, derived_index_build_completed, derived_index_build_scope,
};
use self::packet_planning::{
    choose_index_preparation_strategy, plan_index_packets, plan_index_packets_checked,
    planned_index_definitions,
};
use super::projected_field_values::IndexProjectionSource;
use super::unique_entity_aspect_field_index::{
    rebuild_unique_entity_aspect_field_indexes,
    refresh_unique_entity_aspect_field_index_for_records,
};

pub struct IndexAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

enum IndexBuildProjection<'runtime> {
    Exact(crate::runtime::VisibilityProjectionView<'runtime>),
    Historical(crate::runtime::VisibilityProjectionView<'runtime>),
}

impl<'runtime> IndexBuildProjection<'runtime> {
    fn select(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &'runtime RelationalRuntime,
        branch_id: &crate::history::data::BranchId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<Self>, crate::branch::RelationalBranchBasisDenial> {
        if version_id == runtime.current_version_id() {
            return runtime
                .read_truth()
                .project_branch_head(execution, branch_id, version_id)
                .map(|projection| projection.map(Self::Exact));
        }
        runtime
            .read_truth()
            .try_project_historical_version(execution, version_id)
            .map(|projection| Some(Self::Historical(projection)))
    }

    fn source(&self) -> Option<IndexProjectionSource<'_, 'runtime>> {
        match self {
            Self::Exact(projection) => IndexProjectionSource::exact(projection),
            Self::Historical(projection) => IndexProjectionSource::historical(projection),
        }
    }
}

struct IndexGenerationPublicationBasis {
    source_commit_id: CommitId,
    branch_id: crate::history::data::BranchId,
    version_id: crate::identity::data::VersionId,
    schema_version: crate::schema::data::SchemaVersionId,
}

impl IndexGenerationPublicationBasis {
    fn new(
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

fn publish_index_generations(
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

fn publish_prepared_generation(
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

fn prepared_generation(
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

fn failed_build_outcome(
    source_commit_id: CommitId,
    generations: Vec<DerivedIndexGeneration>,
    failed_indexes: Vec<DerivedIndexId>,
    basis_denial: Option<crate::branch::RelationalBranchBasisDenial>,
) -> DerivedIndexBuildOutcome {
    DerivedIndexBuildOutcome {
        source_commit_id,
        generations,
        failed_indexes,
        basis_denial,
        execution_denial: None,
    }
}

impl RelationalRuntime {
    pub fn index_authority(&self) -> IndexAuthority<'_> {
        IndexAuthority::new(self)
    }
}

impl<'runtime> IndexAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn register(&self, definition: DerivedIndexDefinition) -> DerivedIndexDefinition {
        self.runtime.indexes.register_definition(definition)
    }

    pub(crate) fn refresh_unique_entity_aspect_field_index_for_records(
        &self,
        changed_records: &[crate::transactions::data::RecordRef],
        basis: &crate::mvcc::PreparedIndexRefreshBasis,
    ) {
        refresh_unique_entity_aspect_field_index_for_records(self.runtime, changed_records, basis);
    }

    pub(crate) fn rebuild_unique_entity_aspect_field_indexes(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::branch::RelationalBranchBasisDenial> {
        rebuild_unique_entity_aspect_field_indexes(execution, self.runtime)
    }
}
