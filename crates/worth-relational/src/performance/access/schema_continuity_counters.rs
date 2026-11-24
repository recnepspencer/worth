use super::PerformanceAccess;
use crate::schema::data::SchemaContinuationClassification;
use worth_relational_contracts::internal_port::performance::data::runtime_complexity_counters::RuntimeComplexityCountersOwnerPort as _;

impl PerformanceAccess<'_> {
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
                _ => unreachable!("unhandled Relational contract variant"),
            }
        });
    }

    pub(crate) fn count_descriptor_version_mismatch(&self) {
        self.counters.count(
            crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters::count_descriptor_version_mismatch,
        );
    }
}
