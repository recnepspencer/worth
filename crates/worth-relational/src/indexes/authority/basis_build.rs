use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::{DerivedIndexBuildOutcome, DerivedIndexBuildRequest};

use super::{
    choose_index_preparation_strategy, execute_index_inputs, execute_index_packets,
    failed_build_outcome, index_execution_denial, plan_index_packets, plan_index_packets_checked,
    planned_index_definitions, publish_index_generations,
    record_index_preparation_strategy_counters, IndexAuthority, IndexGenerationPublicationBasis,
    IndexProjectionSource,
};

impl IndexAuthority<'_> {
    /// Build against the exact immutable root already carried by an admitted
    /// owner basis. This path performs no historical retention reacquisition.
    pub fn build_for_basis(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
    ) -> DerivedIndexBuildOutcome {
        self.build_for_basis_inner(request, basis, None)
    }

    pub fn build_for_basis_with_lease(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> DerivedIndexBuildOutcome {
        self.build_for_basis_inner(request, basis, Some(lease))
    }

    fn build_for_basis_inner(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> DerivedIndexBuildOutcome {
        let mut generations = Vec::new();
        let mut failed_indexes = Vec::new();
        let observation = basis.observation();
        let Some(source_commit_id) = observation.commit_id() else {
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                request.index_ids,
                None,
            );
        };
        if request.source_commit_id != source_commit_id {
            return failed_build_outcome(
                request.source_commit_id,
                generations,
                request.index_ids,
                Some(crate::branch::RelationalBranchBasisDenial::MixedAxis(
                    crate::branch::RelationalBranchBasisMismatchAxis::Commit,
                )),
            );
        }
        let source_branch_id = observation.identity().branch_id().clone();
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
        let version_id = observation.version_id();
        let projection = match self.runtime.read_truth().project_observation(&observation) {
            Ok(projection) => projection,
            Err(denial) => {
                return failed_build_outcome(
                    request.source_commit_id,
                    generations,
                    request.index_ids,
                    Some(denial),
                );
            }
        };
        let projection = IndexProjectionSource::exact(&projection)
            .expect("owner-admitted observation projects one exact root");
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
}
