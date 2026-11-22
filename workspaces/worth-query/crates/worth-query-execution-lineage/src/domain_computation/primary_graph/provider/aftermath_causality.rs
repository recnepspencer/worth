//! Pure canonical correction fact record.

use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
};
use worth_relational::facade::{history::RelationalCommitReceipt, transactions::RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCanonicalCorrectionFact {
    pub role: WorthQueryAftermathCausalRole,
    pub parent: RelationalCommitReceipt,
    pub child: RelationalCommitReceipt,
    pub record: RecordRef,
    pub source: WorthQueryCorrectionSourceCoordinates,
    pub root: WorthQueryCorrectionSourceCoordinates,
    pub selected: Option<WorthQueryCorrectionSourceCoordinates>,
    pub active: Option<WorthQueryCorrectionSourceCoordinates>,
    pub prior: Option<WorthQueryCorrectionSourceCoordinates>,
}
