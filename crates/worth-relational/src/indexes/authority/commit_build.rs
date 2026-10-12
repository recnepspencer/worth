use super::*;

impl<'runtime> IndexAuthority<'runtime> {
    /// Build using the allocation authority and lease budget of the caller's request.
    pub fn build_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
    ) -> DerivedIndexBuildOutcome {
        let lease = match execution.allocation_policy() {
            worth_execution::ExecutionAllocationPolicy::Execution(lease) => Some(lease),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation => None,
        };
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

        let selected_projection = match IndexBuildProjection::select(
            execution,
            self.runtime,
            &source_branch_id,
            version_id,
        ) {
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
            Some(inputs) => {
                execute_index_inputs(execution, &projection, inputs, lease, work_budget.as_ref())
            }
            None => execute_index_packets(
                execution,
                &projection,
                plan_index_packets(definitions),
                None,
                None,
            ),
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

    pub(super) fn publish_build_diagnostic(
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
