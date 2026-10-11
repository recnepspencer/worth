use super::*;

impl IndexAccess<'_> {
    pub fn execute_query_plan_with_index_parity(
        &self,
        plan: SnapshotPinnedQueryPlan,
        parity_mode: IndexParityMode,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<IndexParityVerifiedQueryOutcome>, crate::runtime::QueryReadExecutionStop>
    {
        request
            .consult()
            .map_err(crate::runtime::QueryReadExecutionStop::Request)?;
        if plan.packet.access_contract == QueryAccessContract::DerivedIndexWithStorageParity {
            self.runtime
                .performance_access()
                .count_query_index_attempt();
        }
        let storage_execution = || {
            self.runtime
                .read_truth()
                .execute_query_plan(plan.clone(), request)
                .map(|read| read.map(|read| read.outcome))
        };
        let (execution, access_path) = match admissible_access_path(self.runtime, &plan) {
            QueryAccessPath::DerivedIndexGeneration { generation_id } => {
                let Some(index_execution) = execute_index_backed_query_from_generation(
                    self.runtime,
                    &plan,
                    generation_id,
                    request,
                )?
                else {
                    return Ok(None);
                };
                match parity_mode {
                    IndexParityMode::ProductionAdmissibility => {
                        self.runtime.performance_access().count_query_index_path();
                        record_index_execution_counters(self.runtime, &index_execution);
                        (
                            index_execution,
                            QueryAccessPath::DerivedIndexGeneration { generation_id },
                        )
                    }
                    IndexParityMode::SampledParity
                        if !should_verify_sampled_parity(&plan, generation_id) =>
                    {
                        self.runtime.performance_access().count_query_index_path();
                        record_index_execution_counters(self.runtime, &index_execution);
                        (
                            index_execution,
                            QueryAccessPath::DerivedIndexGeneration { generation_id },
                        )
                    }
                    IndexParityMode::SampledParity | IndexParityMode::CertificationParity => {
                        self.runtime
                            .performance_access()
                            .count_query_index_parity_verification();
                        let Some(storage_execution) = storage_execution()? else {
                            return Ok(None);
                        };
                        if storage_execution.result == index_execution.result {
                            self.runtime.performance_access().count_query_index_path();
                            record_index_execution_counters(self.runtime, &index_execution);
                            (
                                index_execution,
                                QueryAccessPath::DerivedIndexGeneration { generation_id },
                            )
                        } else {
                            self.runtime
                                .performance_access()
                                .count_query_index_rejection();
                            record_index_execution_shape(self.runtime, &index_execution);
                            (
                                storage_execution,
                                QueryAccessPath::DerivedIndexRejectedStorageRead {
                                    rejection:
                                        crate::query::data::IndexQueryRejectionClass::CorruptIndexEntries,
                                },
                            )
                        }
                    }
                }
            }
            access_path => {
                if matches!(
                    access_path,
                    QueryAccessPath::DerivedIndexRejectedStorageRead { .. }
                ) {
                    self.runtime
                        .performance_access()
                        .count_query_index_rejection();
                }
                {
                    let Some(storage) = storage_execution()? else {
                        return Ok(None);
                    };
                    (storage, access_path)
                }
            }
        };
        let parity_basis_digest = query_index_parity_basis_digest(
            &access_path,
            parity_mode,
            &execution.result,
            execution.plan.packet.plan_key,
        );
        Ok(Some(IndexParityVerifiedQueryOutcome {
            execution,
            access_path,
            parity_mode,
            parity_basis_digest,
        }))
    }
}
