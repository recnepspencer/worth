mod basis_build;
mod build_execution;
mod diagnostics;
mod maintenance;
mod packet_planning;

use crate::history::data::CommitId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexBuildOutcome, DerivedIndexBuildRequest,
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexGeneration, DerivedIndexGenerationId,
    DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::runtime::RelationalRuntime;

use self::build_execution::{
    execute_index_packets, record_index_preparation_strategy_counters, IndexPreparationResult,
};
use self::diagnostics::{
    derived_index_build_artifact_kind, derived_index_build_completed, derived_index_build_scope,
};
use self::packet_planning::{
    choose_index_preparation_strategy, plan_index_packets, planned_index_definitions,
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
        runtime: &'runtime RelationalRuntime,
        branch_id: &crate::history::data::BranchId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<Self>, crate::branch::RelationalBranchBasisDenial> {
        if version_id == runtime.current_version_id() {
            return runtime
                .read_truth()
                .project_branch_head(branch_id, version_id)
                .map(|projection| projection.map(Self::Exact));
        }
        runtime
            .read_truth()
            .try_project_historical_version(version_id)
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
    results: Vec<IndexPreparationResult>,
    failed_indexes: &mut Vec<DerivedIndexId>,
) -> Vec<DerivedIndexGeneration> {
    let mut generations = Vec::new();
    for result in results {
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
    let generation = DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(runtime.indexes.next_generation_id()),
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
    };
    runtime.indexes.publish_generation(generation.clone());
    generation
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

    pub fn build_for_commit(&self, request: DerivedIndexBuildRequest) -> DerivedIndexBuildOutcome {
        let mut generations = Vec::new();
        let mut failed_indexes = Vec::new();
        let Some((version_id, source_branch_id)) =
            self.source_commit_basis(request.source_commit_id)
        else {
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                request.index_ids,
                None,
            );
        };
        if request.branch_id != source_branch_id {
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                request.index_ids,
                Some(crate::branch::RelationalBranchBasisDenial::MixedAxis(
                    crate::branch::RelationalBranchBasisMismatchAxis::Branch,
                )),
            );
        }

        let (definitions, missing_indexes) =
            planned_index_definitions(self.runtime, &request.index_ids);
        failed_indexes.extend(missing_indexes);

        let strategy = choose_index_preparation_strategy(self.runtime, definitions.len());
        record_index_preparation_strategy_counters(self.runtime, definitions.len(), &strategy);

        let selected_projection =
            match IndexBuildProjection::select(self.runtime, &source_branch_id, version_id) {
                Ok(projection) => projection,
                Err(denial) => {
                    failed_indexes.extend(definitions.iter().map(|definition| definition.index_id));
                    return failed_build_outcome(
                        request.source_commit_id,
                        generations,
                        failed_indexes,
                        Some(denial),
                    );
                }
            };
        let Some(projection) = selected_projection
            .as_ref()
            .and_then(IndexBuildProjection::source)
        else {
            failed_indexes.extend(definitions.iter().map(|definition| definition.index_id));
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                failed_indexes,
                None,
            );
        };
        let Some(schema_version) = projection.schema_version() else {
            failed_indexes.extend(definitions.iter().map(|definition| definition.index_id));
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                failed_indexes,
                None,
            );
        };
        let packets = plan_index_packets(&definitions);
        let results =
            execute_index_packets(self.runtime, &projection, &packets, strategy.selected_mode);

        let publication_basis = IndexGenerationPublicationBasis::new(
            &request,
            source_branch_id,
            version_id,
            schema_version,
        );
        generations = publish_index_generations(
            self.runtime,
            &publication_basis,
            results,
            &mut failed_indexes,
        );

        self.publish_build_diagnostic(
            &request,
            &publication_basis.branch_id,
            &generations,
            &failed_indexes,
        );

        DerivedIndexBuildOutcome {
            source_commit_id: request.source_commit_id,
            generations,
            failed_indexes,
            basis_denial: None,
        }
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
    ) -> Result<(), crate::branch::RelationalBranchBasisDenial> {
        rebuild_unique_entity_aspect_field_indexes(self.runtime)
    }

    fn source_commit_basis(
        &self,
        commit_id: CommitId,
    ) -> Option<(
        crate::identity::data::VersionId,
        crate::history::data::BranchId,
    )> {
        self.runtime
            .history
            .recorded_commit_envelope(commit_id)
            .map(|commit| (commit.commit.version_id, commit.branch_context.clone()))
    }

    fn publish_build_diagnostic(
        &self,
        request: &DerivedIndexBuildRequest,
        branch_id: &crate::history::data::BranchId,
        generations: &[DerivedIndexGeneration],
        failed_indexes: &[DerivedIndexId],
    ) {
        self.runtime
            .publication_authority()
            .push_bounded_diagnostic(
                derived_index_build_scope(),
                derived_index_build_artifact_kind(failed_indexes),
                vec![derived_index_build_completed(
                    request.source_commit_id,
                    branch_id,
                    generations,
                    failed_indexes,
                )],
            );
    }
}
