//! Count the classifications and bridge descriptors produced during preparation.
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::{
    HistoricalInterpretationSensitivity, SchemaContinuationClassification,
    SchemaReconciliationPolicy,
};

pub(super) fn count_schema_transition_classification(
    instrumentation: &RuntimeInstrumentation,
    atoms_inspected: usize,
    changed_subtrees_inspected: usize,
    unchanged_subtrees_reused_by_fingerprint: usize,
) {
    instrumentation.count(|counters| {
        counters.schema_transition_atoms_inspected += atoms_inspected;
        counters.schema_changed_subtrees_inspected += changed_subtrees_inspected;
        counters.schema_unchanged_subtrees_reused_by_fingerprint +=
            unchanged_subtrees_reused_by_fingerprint;
    });
}

pub(super) fn count_schema_bridge_descriptor(
    instrumentation: &RuntimeInstrumentation,
    continuation: SchemaContinuationClassification,
    historical_interpretation: HistoricalInterpretationSensitivity,
    policy: SchemaReconciliationPolicy,
) {
    instrumentation.count(|counters| {
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
            _ => unreachable!("unhandled Relational contract variant"),
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
                counters.schema_reconciliation_permit_lossy_narrowing_with_annotation_count += 1;
            }
            SchemaReconciliationPolicy::RequireExplicitProjection => {
                counters.schema_reconciliation_require_explicit_projection_count += 1;
            }
            _ => unreachable!("unhandled Relational contract variant"),
        }
    });
}
