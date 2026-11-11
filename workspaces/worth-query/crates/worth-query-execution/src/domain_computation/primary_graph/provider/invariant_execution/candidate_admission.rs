//! Each writer admits its own contract under one genuine Native invariant judgment.
use super::*;
enum CoupledContribution<'a> {
    Producer(&'a worth_relational::facade::mvcc::RequiredOutputPublicationMeaning),
    AuthoredStage(usize),
}
pub(super) enum CandidateInvariantJudgment<'borrow, 'candidate> {
    Standalone(&'candidate worth_relational::facade::mvcc::ValidatedRelationalProposal),
    GroupOwner(&'borrow ClosedCoupledInvariantReceipts<'candidate>),
    GroupMember(&'borrow ClosedCoupledInvariantReceipts<'candidate>),
}
impl<'candidate> CandidateInvariantJudgment<'_, 'candidate> {
    fn candidate(&self) -> &'candidate worth_relational::facade::mvcc::ValidatedRelationalProposal {
        match self {
            Self::Standalone(candidate) => candidate,
            Self::GroupOwner(receipts) | Self::GroupMember(receipts) => receipts.candidate(),
        }
    }
}
impl WorthQueryPrimaryGraphProvider {
    pub(super) fn admit_candidate_material(
        &self,
        judgment: CandidateInvariantJudgment<'_, '_>,
        material: &ApplicationInvariantCandidateMaterial,
        touches: impl FnOnce() -> Result<
            super::super::application_touch_admission::WorthQueryInstalledTouchAdmissionEvidence,
            WorthQueryInvariantExecutionFailure,
        >,
    ) -> Result<
        super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
        WorthQueryInvariantExecutionFailure,
    > {
        let semantic_work = u64::try_from(material.semantic.expected.len()).map_err(|_| {
            closure_failure("candidate semantic cardinality exceeds work representation")
        })?;
        let candidate = judgment.candidate();
        let owner_work = match &judgment {
            CandidateInvariantJudgment::Standalone(_) => receipt_closure::validate_receipt_closure(
                candidate.invariant_evidence(),
                &material.requirements,
                semantic_work,
            )?,
            CandidateInvariantJudgment::GroupOwner(receipts) => semantic_work
                .checked_add(receipts.native_work())
                .ok_or_else(owner_failure)?,
            CandidateInvariantJudgment::GroupMember(_) => semantic_work,
        };
        super::super::super::product_operation::admit_required_invariants(
            candidate
                .invariant_evidence()
                .custom_invariant_execution_receipts(),
            material.producer_required_invariants,
        )
        .map_err(|denial| {
            WorthQueryInvariantExecutionFailure::new(
                WorthQueryInvariantExecutionDenialKind::ProducerRequirementNotAdmitted(denial),
                "producer-required invariant did not admit the publication",
            )
        })?;
        let touch_admission = touches()?;
        let summary = candidate.invariant_evidence().summary();
        let owns_native_judgment = !matches!(judgment, CandidateInvariantJudgment::GroupMember(_));
        Ok(
            super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters::new(
                WorthQueryInvariantWorkMint { _private: () },
                material.decision_facts,
                material.semantic.expected.len(),
                material.semantic.expected.len(),
                owner_work,
                if owns_native_judgment {
                    summary.execution_count
                } else {
                    0
                },
                if owns_native_judgment {
                    summary.result_count
                } else {
                    0
                },
                material.expected_step_preparation_work,
                touch_admission,
            ),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn coupled_invariant_requirements(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Result<
        Vec<worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement>,
        WorthQueryInvariantExecutionFailure,
    > {
        execution.consult().map_err(|cause| {
            super::relational_validation::context_execution_failure(cause.into())
        })?;
        let attempts = self.attempts.lock().unwrap_or_else(|p| p.into_inner());
        let staged = attempts
            .staged_attempt(session)
            .ok_or_else(missing_candidate_failure)?;
        Ok(staged.invariant_requirements().to_vec())
    }

    pub(in crate::domain_computation::primary_graph) fn admit_coupled_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        receipts: &ClosedCoupledInvariantReceipts<'_>,
        meaning: &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        self.admit_coupled_part(
            execution,
            session,
            receipts,
            CoupledContribution::Producer(meaning),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn admit_authored_stage_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        receipts: &ClosedCoupledInvariantReceipts<'_>,
        stage: usize,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        self.admit_coupled_part(
            execution,
            session,
            receipts,
            CoupledContribution::AuthoredStage(stage),
        )
    }

    fn admit_coupled_part(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        receipts: &ClosedCoupledInvariantReceipts<'_>,
        contribution: CoupledContribution<'_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let material = self.invariant_candidate_material(session, None)?;
        let candidate = receipts.candidate();
        let work = self.admit_candidate_material(CandidateInvariantJudgment::GroupMember(receipts), &material, || {
            // Native projects this writer's own part only; the other parts are theirs.
            let touches = match contribution {
                CoupledContribution::Producer(meaning) => candidate.coupled_producer_mutation_touches(execution, meaning),
                CoupledContribution::AuthoredStage(stage) => candidate.authored_stage_mutation_touches(execution, stage),
            }
                .ok_or_else(missing_candidate_failure)?
                .map_err(super::super::application_touch_admission::TouchAdmissionFailure::Projection)
                .map_err(super::super::application_touch_admission::TouchAdmissionFailure::into_invariant_failure)?;
            super::super::application_touch_admission::admit_coupled_contribution_touches(
                execution, &touches, &self.graph.layout, &material.application_graph_reads,
                &material.application_touches, &material.application_read_touch_overlap,
            ).map_err(super::super::application_touch_admission::TouchAdmissionFailure::into_invariant_failure)
        })?;
        self.attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retain_coupled_invariant_approved(
                session,
                candidate.invariant_evidence().clone(),
                work,
            )
            .map_err(closure_failure)
    }
}
