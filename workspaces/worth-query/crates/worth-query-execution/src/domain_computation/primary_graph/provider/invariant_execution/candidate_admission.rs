//! Each writer admits its own contract under one genuine Native invariant judgment.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub(super) fn admit_candidate_material(
        &self,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
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
        let owner_work = receipt_closure::validate_receipt_closure(
            candidate.invariant_evidence(),
            &material.requirements,
            semantic_work,
        )?;
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
        Ok(
            super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters::new(
                WorthQueryInvariantWorkMint { _private: () },
                material.decision_facts,
                material.semantic.expected.len(),
                material.semantic.expected.len(),
                owner_work,
                summary.execution_count,
                summary.result_count,
                material.expected_step_preparation_work,
                touch_admission,
            ),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn admit_coupled_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
        meaning: &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let material = self.invariant_candidate_material(session)?;
        let work = self.admit_candidate_material(candidate, &material, || {
            let mut selected = None;
            for (_, actual, touches) in candidate.coupled_producer_mutation_touches(execution) {
                if actual == meaning { selected = Some(touches); }
            }
            let touches = selected.ok_or_else(missing_candidate_failure)?
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
