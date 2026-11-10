//! One proposed-state pass can suspend after its cheap Native prerequisites.
use super::*;
pub(crate) struct PendingProposedPass {
    pub(super) prefix: ReducedInvariantPrefix,
    pub(super) proposal: crate::mvcc::RelationalMutationProposalIdentity,
}
pub(crate) enum StartedProposedPass {
    Pending(PendingProposedPass),
    Refused(InvariantExecutionResult),
}
impl InvariantEngine<'_> {
    pub(crate) fn begin_proposed<'state>(
        &self,
        execution: &worth_execution::ExecutionRequest<'state, 'state>,
        request: InvariantExecutionRequest<'state>,
        prepare_sources: &mut dyn FnMut() -> Result<
            (),
            crate::transactions::data::TransactionCommitError,
        >,
    ) -> Result<StartedProposedPass, crate::transactions::data::TransactionCommitError> {
        check_live(*execution)?;
        self.admit_custom_posture(execution, &request)?;
        if execution.has_accounting() {
            return execution
                .run(ExecutionWorkCeiling::new(execution.work_ceiling()), |_| {
                    self.begin_proposed_inner(execution, request, prepare_sources)
                })
                .map(|(result, _)| result)
                .map_err(request_failure)?;
        }
        self.begin_proposed_inner(execution, request, prepare_sources)
    }
    fn begin_proposed_inner<'state>(
        &self,
        execution: &worth_execution::ExecutionRequest<'state, 'state>,
        request: InvariantExecutionRequest<'state>,
        prepare_sources: &mut dyn FnMut() -> Result<
            (),
            crate::transactions::data::TransactionCommitError,
        >,
    ) -> Result<StartedProposedPass, crate::transactions::data::TransactionCommitError> {
        let proposal = request.proposal_identity().cloned().ok_or_else(|| {
            crate::transactions::data::TransactionCommitError::conflict(
                crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::StaleValidationBasis {
                        detail: "native prerequisites require an exact proposal identity".into(),
                    },
                ),
            )
        })?;
        let lease = crate::authority::commit::pipeline::request_lease(execution);
        let mut runtime = self.runtime.with_candidate_input_plan(&request);
        runtime.custom_kernel_posture = lease.map_or(
            worth_foundational::ExecutionPosture::Serial,
            ExecutionResourceLease::resolved_posture,
        );
        let mut prefix = ReducedInvariantPrefix::new();
        if request.should_execute_anything()
            && let Some(stage) = super::super::stage_execution::execute_stage(
                &runtime,
                *execution,
                &request,
                lease,
                InvariantStage::Native,
            )?
        {
            prefix = prefix.absorb_in_request(
                stage,
                *execution,
                lease,
                self.runtime.commit_work_budget.as_ref(),
            )?;
        }
        check_live(*execution)?;
        // The commit-boundary Native prefix admits Source preparation and its readers together.
        let native_admitted = !prefix.has_blocking_violation();
        if native_admitted {
            prepare_sources()?;
        }
        if native_admitted && request.should_execute_anything() {
            let selection = InvariantStage::Custom {
                packet_offset: prefix.packet_count(),
            };
            if let Some(stage) = super::super::stage_execution::execute_stage(
                &runtime, *execution, &request, lease, selection,
            )? {
                prefix = prefix.absorb_in_request(
                    stage,
                    *execution,
                    lease,
                    runtime.commit_work_budget.as_ref(),
                )?;
            }
        }
        if let Some(inputs) = runtime.shared_candidate_inputs() {
            runtime
                .performance_access()
                .count_custom_invariant_candidate_inputs(inputs.counters());
        }
        check_live(*execution)?;
        if prefix.has_blocking_violation() {
            let result = prefix
                .finish_in_request(
                    &request,
                    *execution,
                    lease,
                    self.runtime.commit_work_budget.as_ref(),
                )?
                .into_result();
            return Ok(StartedProposedPass::Refused(result));
        }
        Ok(StartedProposedPass::Pending(PendingProposedPass {
            prefix,
            proposal,
        }))
    }
}
