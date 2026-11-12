use super::*;
use crate::speculation::BridgePreviewReplayBundle;

impl BridgeDiagnosticsFacade {
    pub fn explain_route_record(&self, record: &BridgeRouteRecord) -> BridgeRouteExplanation {
        BridgeRouteExplanation::from_route_record(record)
    }

    pub fn explain_last_route_record(&self) -> Option<BridgeRouteExplanation> {
        self.last_route_record()
            .map(|record| BridgeRouteExplanation::from_route_record(&record))
    }

    pub fn explain_continuity_record(
        &self,
        record: &BridgeCanonicalContinuityRecord,
    ) -> BridgeContinuityExplanation {
        BridgeContinuityExplanation::from_canonical_record(record)
    }

    pub fn explain_merge_record(
        &self,
        record: &BridgeCanonicalMergeRecord,
    ) -> BridgeMergeExplanation {
        BridgeMergeExplanation::from_canonical_record(record)
    }

    pub fn explain_last_continuity_record(&self) -> Option<BridgeContinuityExplanation> {
        self.last_canonical_continuity_record()
            .map(|record| BridgeContinuityExplanation::from_canonical_record(&record))
    }

    pub fn explain_last_merge_record(&self) -> Option<BridgeMergeExplanation> {
        self.last_merge_record()
            .map(|record| BridgeMergeExplanation::from_canonical_record(&record))
    }

    pub fn explain_bulk_record(
        &self,
        record: &BridgeCanonicalBulkPlanRecord,
    ) -> BridgeBulkPlanExplanation {
        BridgeBulkPlanExplanation::from_canonical_record(record)
    }

    pub fn explain_last_bulk_record(&self) -> Option<BridgeBulkPlanExplanation> {
        self.last_bulk_record()
            .map(|record| BridgeBulkPlanExplanation::from_canonical_record(&record))
    }

    pub fn explain_historical_evaluation_record(
        &self,
        record: &BridgeCanonicalHistoricalEvaluationRecord,
    ) -> BridgeHistoricalEvaluationExplanation {
        BridgeHistoricalEvaluationExplanation::from_canonical_record(record)
    }

    pub fn explain_last_historical_evaluation_record(
        &self,
    ) -> Option<BridgeHistoricalEvaluationExplanation> {
        self.last_historical_evaluation_record()
            .map(|record| BridgeHistoricalEvaluationExplanation::from_canonical_record(&record))
    }

    pub fn explain_source_materialization_record(
        &self,
        record: &crate::source::SourceMaterializationRecord,
    ) -> crate::diagnostics::BridgeSourceMaterializationExplanation {
        crate::diagnostics::BridgeSourceMaterializationExplanation::from_record(record)
    }

    pub fn explain_last_source_materialization_record(
        &self,
    ) -> Option<crate::diagnostics::BridgeSourceMaterializationExplanation> {
        self.last_source_materialization_record().map(|record| {
            crate::diagnostics::BridgeSourceMaterializationExplanation::from_record(&record)
        })
    }

    pub fn explain_source_failure_record(
        &self,
        record: &crate::source::SourceFailureRecord,
    ) -> crate::diagnostics::BridgeSourceFailureExplanation {
        crate::diagnostics::BridgeSourceFailureExplanation::from_record(record)
    }

    pub fn explain_last_source_failure_record(
        &self,
    ) -> Option<crate::diagnostics::BridgeSourceFailureExplanation> {
        self.last_source_failure_record()
            .map(|record| crate::diagnostics::BridgeSourceFailureExplanation::from_record(&record))
    }

    pub fn explain_structural_remap_record(
        &self,
        record: &crate::diagnostics::BridgeCanonicalStructuralRemapRecord,
    ) -> crate::diagnostics::BridgeStructuralRemapExplanation {
        crate::diagnostics::BridgeStructuralRemapExplanation::from_canonical_record(record)
    }

    pub fn explain_last_structural_remap_record(
        &self,
    ) -> Option<crate::diagnostics::BridgeStructuralRemapExplanation> {
        self.last_structural_remap_record().map(|record| {
            crate::diagnostics::BridgeStructuralRemapExplanation::from_canonical_record(&record)
        })
    }

    pub fn explain_structural_branch_comparison_record(
        &self,
        record: &crate::diagnostics::BridgeCanonicalStructuralBranchComparisonRecord,
    ) -> crate::diagnostics::BridgeStructuralBranchComparisonExplanation {
        crate::diagnostics::BridgeStructuralBranchComparisonExplanation::from_canonical_record(
            record,
        )
    }

    pub fn explain_last_structural_branch_comparison_record(
        &self,
    ) -> Option<crate::diagnostics::BridgeStructuralBranchComparisonExplanation> {
        self.last_structural_branch_comparison_record().map(|record| {
            crate::diagnostics::BridgeStructuralBranchComparisonExplanation::from_canonical_record(
                &record,
            )
        })
    }

    pub fn explain_stream_checkpoint(
        &self,
        checkpoint: &ConsumerCheckpointToken,
    ) -> BridgeStreamCheckpointExplanation {
        BridgeStreamCheckpointExplanation::from_checkpoint(checkpoint)
    }

    pub fn explain_last_stream_checkpoint(&self) -> Option<BridgeStreamCheckpointExplanation> {
        self.last_stream_checkpoint()
            .map(|checkpoint| BridgeStreamCheckpointExplanation::from_checkpoint(&checkpoint))
    }

    pub fn explain_stream_replay_record(
        &self,
        record: &CanonicalStreamReplayRecord,
    ) -> BridgeStreamReplayExplanation {
        BridgeStreamReplayExplanation::from_replay_record(record)
    }

    pub fn explain_last_stream_replay_record(&self) -> Option<BridgeStreamReplayExplanation> {
        self.last_stream_replay_record()
            .map(|record| BridgeStreamReplayExplanation::from_replay_record(&record))
    }

    pub fn explain_preview_execution_record(
        &self,
        record: &BridgePreviewExecutionRecord,
    ) -> BridgePreviewExecutionExplanation {
        BridgePreviewExecutionExplanation::from_record(record)
    }

    pub fn explain_last_preview_execution_record(
        &self,
    ) -> Option<BridgePreviewExecutionExplanation> {
        self.last_preview_execution_record()
            .map(|record| BridgePreviewExecutionExplanation::from_record(&record))
    }

    pub fn explain_preview_discard_record(
        &self,
        record: &BridgePreviewDiscardRecord,
    ) -> BridgePreviewDiscardExplanation {
        BridgePreviewDiscardExplanation::from_record(record)
    }

    pub fn explain_last_preview_discard_record(&self) -> Option<BridgePreviewDiscardExplanation> {
        self.last_preview_discard_record()
            .map(|record| BridgePreviewDiscardExplanation::from_record(&record))
    }

    pub fn explain_preview_promotion_record(
        &self,
        record: &BridgePreviewPromotionRecord,
    ) -> BridgePreviewPromotionExplanation {
        BridgePreviewPromotionExplanation::from_record(record)
    }

    pub fn explain_last_preview_promotion_record(
        &self,
    ) -> Option<BridgePreviewPromotionExplanation> {
        self.last_preview_promotion_record()
            .map(|record| BridgePreviewPromotionExplanation::from_record(&record))
    }

    pub fn explain_preview_replay_bundle(
        &self,
        bundle: &BridgePreviewReplayBundle,
    ) -> BridgePreviewReplayExplanation {
        BridgePreviewReplayExplanation::from_bundle(bundle)
    }

    pub fn explain_writeback_candidate(
        &self,
        candidate: &crate::writeback::BridgeValidatedWritebackCandidate,
    ) -> BridgeWritebackCandidateExplanation {
        BridgeWritebackCandidateExplanation::from_candidate(candidate)
    }

    pub fn explain_writeback_loop_prevention(
        &self,
        report: &crate::writeback::BridgeWritebackLoopPreventionReport,
    ) -> BridgeWritebackLoopPreventionExplanation {
        BridgeWritebackLoopPreventionExplanation::from_report(report)
    }

    pub fn explain_writeback_strategy_coherence(
        &self,
        report: &crate::writeback::BridgeWritebackStrategyCoherenceReport,
    ) -> BridgeWritebackStrategyCoherenceExplanation {
        BridgeWritebackStrategyCoherenceExplanation::from_report(report)
    }

    pub fn explain_writeback_outcome(
        &self,
        outcome: &crate::writeback::BridgeWritebackAuthorityOutcome,
    ) -> BridgeWritebackOutcomeExplanation {
        BridgeWritebackOutcomeExplanation::from_outcome(outcome)
    }

    pub fn explain_writeback_replay_bundle(
        &self,
        bundle: &crate::writeback::BridgeWritebackReplayBundle,
    ) -> BridgeWritebackReplayExplanation {
        BridgeWritebackReplayExplanation::from_bundle(bundle)
    }

    pub fn explain_writeback_admission_record(
        &self,
        record: &crate::writeback::BridgeWritebackFamilyAdmissionRecord,
    ) -> BridgeWritebackAdmissionExplanation {
        BridgeWritebackAdmissionExplanation::from_record(record)
    }

    pub fn explain_last_writeback_admission_record(
        &self,
    ) -> Option<BridgeWritebackAdmissionExplanation> {
        self.last_writeback_admission_record()
            .map(|record| BridgeWritebackAdmissionExplanation::from_record(&record))
    }

    pub fn explain_writeback_execution_record(
        &self,
        record: &crate::writeback::BridgeWritebackExecutionRecord,
    ) -> BridgeWritebackExecutionExplanation {
        BridgeWritebackExecutionExplanation::from_record(record)
    }

    pub fn explain_last_writeback_execution_record(
        &self,
    ) -> Result<
        BridgeWritebackExecutionExplanation,
        worth_foundational::facade::DiagnosticRecordAbsence,
    > {
        self.last_writeback_execution_record()
            .map(|record| BridgeWritebackExecutionExplanation::from_record(&record))
    }

    pub fn explain_writeback_mapper_record(
        &self,
        record: &crate::writeback::BridgeWritebackMapperRecord,
    ) -> BridgeWritebackMapperExplanation {
        BridgeWritebackMapperExplanation::from_record(record)
    }

    pub fn explain_last_writeback_mapper_record(&self) -> Option<BridgeWritebackMapperExplanation> {
        self.last_writeback_mapper_record()
            .map(|record| BridgeWritebackMapperExplanation::from_record(&record))
    }

    pub fn explain_writeback_mapper_envelope(
        &self,
        envelope: &crate::writeback::BridgeWritebackMapperEnvelope,
    ) -> BridgeWritebackMapperEnvelopeExplanation {
        BridgeWritebackMapperEnvelopeExplanation::from_envelope(envelope)
    }

    pub fn explain_last_writeback_mapper_envelope(
        &self,
    ) -> Option<BridgeWritebackMapperEnvelopeExplanation> {
        self.last_writeback_mapper_envelope()
            .map(|envelope| BridgeWritebackMapperEnvelopeExplanation::from_envelope(&envelope))
    }

    pub fn explain_writeback_mapped_family_input(
        &self,
        mapped_input: &crate::writeback::BridgeMappedWritebackFamilyInput,
    ) -> BridgeMappedWritebackFamilyInputExplanation {
        BridgeMappedWritebackFamilyInputExplanation::from_mapped_input(mapped_input)
    }

    pub fn explain_last_writeback_mapped_family_input(
        &self,
    ) -> Option<BridgeMappedWritebackFamilyInputExplanation> {
        self.last_writeback_mapped_family_input()
            .map(|mapped_input| {
                BridgeMappedWritebackFamilyInputExplanation::from_mapped_input(&mapped_input)
            })
    }

    pub fn explain_writeback_replay_record(
        &self,
        record: &crate::writeback::BridgeWritebackReplayRecord,
    ) -> BridgeWritebackReplayRecordExplanation {
        BridgeWritebackReplayRecordExplanation::from_record(record)
    }

    pub fn explain_last_writeback_replay_record(
        &self,
    ) -> Result<
        BridgeWritebackReplayRecordExplanation,
        worth_foundational::facade::DiagnosticRecordAbsence,
    > {
        self.last_writeback_replay_record()
            .map(|record| BridgeWritebackReplayRecordExplanation::from_record(&record))
    }

    pub fn last_canonical_route_record(&self) -> Option<BridgeCanonicalRouteRecord> {
        self.last_route_record()
            .map(BridgeCanonicalRouteRecord::from_route_record)
    }

    pub fn handle(&self) -> BridgeDiagnosticsHandle {
        BridgeDiagnosticsHandle {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
        }
    }
}
