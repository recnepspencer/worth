use super::UiHostObservationReportValidation;
use worth_ui_host_contract::UiHostObservationCanonicalCore;

impl UiHostObservationReportValidation {
    /// Release delivered reports, preserving the sequence frontier and bounded
    /// duplicate fingerprints. Retention is pending delivery, not input history.
    pub(crate) fn retire_delivered_batch(&mut self, core: UiHostObservationCanonicalCore) {
        let Some(partition) = self.partitions.get_mut(&core.binding()) else {
            return;
        };
        let sequences = core.sequences();
        partition.reports.retain(|retained| {
            let sequence = retained.report.report().sequence();
            if sequence < sequences.first() || sequence > sequences.last() {
                return true;
            }
            partition.byte_count -= retained.encoded_len;
            self.global_reports -= 1;
            self.global_bytes -= retained.encoded_len;
            let basis = self
                .observation_bases
                .get_mut(&retained.frame)
                .expect("retained report owns its observation basis");
            basis.retained_reports -= 1;
            if basis.retained_reports == 0 {
                self.observation_bases.remove(&retained.frame);
            }
            false
        });
    }
}
