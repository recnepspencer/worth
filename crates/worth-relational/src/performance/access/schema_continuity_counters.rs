use super::PerformanceAccess;
use crate::schema::data::{
    HistoricalInterpretationSensitivity, SchemaContinuationClassification,
    SchemaReconciliationPolicy,
};

impl PerformanceAccess<'_> {
    pub(crate) fn count_schema_transition_classification(
        &self,
        atoms_inspected: usize,
        changed_subtrees_inspected: usize,
        unchanged_subtrees_reused_by_fingerprint: usize,
    ) {
        self.counters.count(|counters| {
            counters.schema_transition_atoms_inspected += atoms_inspected;
            counters.schema_changed_subtrees_inspected += changed_subtrees_inspected;
            counters.schema_unchanged_subtrees_reused_by_fingerprint +=
                unchanged_subtrees_reused_by_fingerprint;
        });
    }

    pub(crate) fn count_schema_bridge_descriptor(
        &self,
        continuation: SchemaContinuationClassification,
        historical_interpretation: HistoricalInterpretationSensitivity,
        policy: SchemaReconciliationPolicy,
    ) {
        self.counters.count(|counters| {
            counters.schema_bridge_descriptors_built += 1;
            match continuation {
                SchemaContinuationClassification::ContinueUnchanged => {
                    counters.schema_transition_continue_unchanged_count += 1;
                }
                SchemaContinuationClassification::ContinueWithTransparentBridge => {
                    counters.schema_transition_continue_transparent_bridge_count += 1;
                }
                SchemaContinuationClassification::ContinueWithVisibleBridge => {
                    counters.schema_transition_continue_visible_bridge_count += 1;
                }
                SchemaContinuationClassification::ContinueWithContractUpgrade => {
                    counters.schema_transition_continue_contract_upgrade_count += 1;
                }
                SchemaContinuationClassification::RequireRenegotiation => {
                    counters.schema_transition_require_renegotiation_count += 1;
                }
                SchemaContinuationClassification::Rejected => {
                    counters.schema_transition_rejected_count += 1;
                }
            }
            if historical_interpretation != HistoricalInterpretationSensitivity::NotSensitive {
                counters.schema_historical_interpretation_sensitive_boundaries += 1;
            }
            match policy {
                SchemaReconciliationPolicy::RejectLossyNarrowing => {
                    counters.schema_reconciliation_reject_lossy_narrowing_count += 1;
                }
                SchemaReconciliationPolicy::PreserveInformation => {
                    counters.schema_reconciliation_preserve_information_count += 1;
                }
                SchemaReconciliationPolicy::PreserveTargetContract => {
                    counters.schema_reconciliation_preserve_target_contract_count += 1;
                }
                SchemaReconciliationPolicy::PreserveSourceContract => {
                    counters.schema_reconciliation_preserve_source_contract_count += 1;
                }
                SchemaReconciliationPolicy::PermitLossyNarrowingWithAnnotation => {
                    counters.schema_reconciliation_permit_lossy_narrowing_with_annotation_count +=
                        1;
                }
                SchemaReconciliationPolicy::RequireExplicitProjection => {
                    counters.schema_reconciliation_require_explicit_projection_count += 1;
                }
            }
        });
    }

    pub(crate) fn count_schema_normalized_descriptor_composition(&self, count: usize) {
        self.counters.count(|counters| {
            counters.schema_normalized_descriptor_compositions += count;
        });
    }

    pub(crate) fn count_subscriber_resume_evaluation(
        &self,
        outcome: SchemaContinuationClassification,
    ) {
        self.counters.count(|counters| {
            counters.subscriber_resume_evaluations += 1;
            match outcome {
                SchemaContinuationClassification::ContinueUnchanged => {
                    counters.subscriber_continue_unchanged_count += 1;
                }
                SchemaContinuationClassification::ContinueWithTransparentBridge => {
                    counters.subscriber_continue_transparent_bridge_count += 1;
                }
                SchemaContinuationClassification::ContinueWithVisibleBridge => {
                    counters.subscriber_continue_visible_bridge_count += 1;
                }
                SchemaContinuationClassification::ContinueWithContractUpgrade => {
                    counters.subscriber_continue_contract_upgrade_count += 1;
                }
                SchemaContinuationClassification::RequireRenegotiation => {
                    counters.subscriber_require_renegotiation_count += 1;
                }
                SchemaContinuationClassification::Rejected => {
                    counters.subscriber_rejected_count += 1;
                }
            }
        });
    }

    pub(crate) fn count_descriptor_version_mismatch(&self) {
        self.counters.count(|counters| {
            counters.descriptor_version_mismatches_encountered += 1;
        });
    }
}
