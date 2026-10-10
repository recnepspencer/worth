use crate::identity::hash_parts;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey;

use super::{
    allowed_phase_six_synthetic_seams, worth_query_lower_runtime_representative_surface,
    WorthQueryLowerRuntimeRepresentativeEvidenceSource,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSyntheticTailRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    justification: &'static str,
    evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource,
}

impl WorthQueryLowerRuntimeSyntheticTailRow {
    fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        justification: &'static str,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource,
    ) -> Self {
        Self {
            seam_key,
            justification,
            evidence_source,
        }
    }

    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn justification(&self) -> &'static str {
        self.justification
    }

    pub fn evidence_source(&self) -> WorthQueryLowerRuntimeRepresentativeEvidenceSource {
        self.evidence_source
    }

    fn row_digest(&self) -> String {
        hash_parts(&[
            self.seam_key.as_str().to_string(),
            self.justification.to_string(),
            evidence_source_label(self.evidence_source).to_string(),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSyntheticTailReport {
    rows: Vec<WorthQueryLowerRuntimeSyntheticTailRow>,
    report_digest: String,
    justification_digest: String,
}

impl WorthQueryLowerRuntimeSyntheticTailReport {
    fn new(rows: Vec<WorthQueryLowerRuntimeSyntheticTailRow>) -> Self {
        let report_digest = hash_parts(
            &rows
                .iter()
                .map(WorthQueryLowerRuntimeSyntheticTailRow::row_digest)
                .collect::<Vec<_>>(),
        );
        let justification_digest = hash_parts(
            &rows
                .iter()
                .map(|row| format!("{}|{}", row.seam_key.as_str(), row.justification))
                .collect::<Vec<_>>(),
        );
        Self {
            rows,
            report_digest,
            justification_digest,
        }
    }

    pub fn rows(&self) -> &[WorthQueryLowerRuntimeSyntheticTailRow] {
        &self.rows
    }

    pub fn report_digest(&self) -> &str {
        &self.report_digest
    }

    pub fn justification_digest(&self) -> &str {
        &self.justification_digest
    }
}

pub fn worth_query_lower_runtime_synthetic_tail_report(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryLowerRuntimeSyntheticTailReport {
    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let rows = allowed_phase_six_synthetic_seams()
        .iter()
        .map(|row| {
            let evidence_source = surface
                .evidence_source_for(row.seam_key())
                .unwrap_or_else(|| panic!("missing synthetic seam {}", row.seam_key().as_str()));
            assert_eq!(
                evidence_source,
                WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
                "synthetic-tail row {} must remain inventory-synthesized",
                row.seam_key().as_str()
            );
            WorthQueryLowerRuntimeSyntheticTailRow::new(
                row.seam_key(),
                row.justification(),
                evidence_source,
            )
        })
        .collect::<Vec<_>>();

    WorthQueryLowerRuntimeSyntheticTailReport::new(rows)
}

fn evidence_source_label(
    source: WorthQueryLowerRuntimeRepresentativeEvidenceSource,
) -> &'static str {
    match source {
        WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture => {
            "runtime-backed-fixture"
        }
        WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized => {
            "inventory-synthesized"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_tail_report_matches_allowlist_exactly() {
        let host_request = crate::bridge_host_request::serial_request();
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let report = worth_query_lower_runtime_synthetic_tail_report(resource_request);

        assert_eq!(
            report.rows().len(),
            allowed_phase_six_synthetic_seams().len()
        );
        for allowed in allowed_phase_six_synthetic_seams() {
            assert!(report
                .rows()
                .iter()
                .any(|row| row.seam_key() == allowed.seam_key()));
        }
    }

    #[test]
    fn synthetic_tail_report_rows_stay_inventory_synthesized() {
        let host_request = crate::bridge_host_request::serial_request();
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let report = worth_query_lower_runtime_synthetic_tail_report(resource_request);

        for row in report.rows() {
            assert_eq!(
                row.evidence_source(),
                WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized
            );
        }
    }
}
