use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod lanes;
mod rejections;
mod rows;

use lanes::{
    branch_diff_lane, branch_lane, current_historical_diff_lane, current_lane, historical_lane,
    preview_lane, store_historical_lane,
};
use rows::{canonical_row, rejection_row};

use super::lane::HistoricalDiffCertificationMatrix;
use super::row_catalog::{
    HISTORICAL_DIFF_CANONICAL_ROW_SPECS, HISTORICAL_DIFF_REJECTION_ROW_SPECS,
};
pub struct MilestoneSixHistoricalDiffCertificationAdapter;

impl MilestoneSixHistoricalDiffCertificationAdapter {
    pub fn branch_scoped_historical_and_diff_query_context_test(
    ) -> Result<HistoricalDiffCertificationMatrix, IdentityDerivationDenial> {
        let current = current_lane();
        let branch = branch_lane();
        let historical = historical_lane()?;
        let store_historical = store_historical_lane()?;
        let preview = preview_lane();
        let branch_diff = branch_diff_lane();
        let current_historical_diff = current_historical_diff_lane()?;

        Ok(HistoricalDiffCertificationMatrix {
            suite_name: "Historical / Diff / Basis Parity Test",
            rows: HISTORICAL_DIFF_CANONICAL_ROW_SPECS
                .iter()
                .map(|spec| {
                    canonical_row(
                        spec,
                        &current,
                        &branch,
                        &historical,
                        &store_historical,
                        &preview,
                        &branch_diff,
                        &current_historical_diff,
                    )
                })
                .collect(),
            rejection_rows: HISTORICAL_DIFF_REJECTION_ROW_SPECS
                .iter()
                .map(rejection_row)
                .collect(),
        })
    }
}
