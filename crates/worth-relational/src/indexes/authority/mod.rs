mod basis_build;
mod build_execution;
mod diagnostics;
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

    pub fn build_for_commit(&self, request: DerivedIndexBuildRequest) -> DerivedIndexBuildOutcome {
        self.build_for_commit_inner(request, None)
    }

    pub fn build_for_commit_with_lease(
        &self,
        request: DerivedIndexBuildRequest,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> DerivedIndexBuildOutcome {
        self.build_for_commit_inner(request, Some(lease))
    }

    fn build_for_commit_inner(
        &self,
        request: DerivedIndexBuildRequest,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> DerivedIndexBuildOutcome {
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

        let work_budget = lease.map(|_| crate::execution::RequestWorkBudget::new());
        let (definitions, checked_inputs, present_indexes, missing_indexes) =
            if let Some(lease) = lease {
                match plan_index_packets_checked(
                    self.runtime,
                    &request.index_ids,
                    lease,
                    work_budget.as_ref().expect("leased index work account"),
                ) {
                    Ok(planned) => (
                        Vec::new(),
                        Some(planned.inputs),
                        planned.present,
                        planned.missing,
                    ),
                    Err(stop) => {
                        return DerivedIndexBuildOutcome {
                            source_commit_id: request.source_commit_id,
                            generations,
                            failed_indexes: request.index_ids,
                            basis_denial: None,
                            execution_denial: Some(index_execution_denial(stop)),
                        };
                    }
                }
            } else {
                let (definitions, missing) =
                    planned_index_definitions(self.runtime, &request.index_ids);
                let present: Vec<_> = definitions
                    .iter()
                    .map(|definition| definition.index_id)
                    .collect();
                (definitions, None, present, missing)
            };
        failed_indexes.extend(missing_indexes);

        let strategy =
            choose_index_preparation_strategy(self.runtime, lease, present_indexes.len());
        record_index_preparation_strategy_counters(self.runtime, present_indexes.len(), &strategy);

        let selected_projection =
            match IndexBuildProjection::select(self.runtime, &source_branch_id, version_id) {
                Ok(projection) => projection,
                Err(denial) => {
                    failed_indexes.extend(present_indexes.iter().copied());
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
            failed_indexes.extend(present_indexes.iter().copied());
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                failed_indexes,
                None,
            );
        };
        let Some(schema_version) = projection.schema_version() else {
            failed_indexes.extend(present_indexes.iter().copied());
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                failed_indexes,
                None,
            );
        };
        let results = match checked_inputs {
            Some(inputs) => execute_index_inputs(&projection, inputs, lease, work_budget.as_ref()),
            None => execute_index_packets(&projection, plan_index_packets(definitions), None, None),
        };
        let results = match results {
            Ok(results) => results,
            Err(execution_denial) => {
                failed_indexes.extend(present_indexes.iter().copied());
                return DerivedIndexBuildOutcome {
                    source_commit_id: request.source_commit_id,
                    generations,
                    failed_indexes,
                    basis_denial: None,
                    execution_denial: Some(execution_denial),
                };
            }
        };

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
            execution_denial: None,
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
