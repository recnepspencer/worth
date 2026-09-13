use crate::branch::{
    ProductBranchObservation, ProductBranchObservationMismatch, ProductBranchReferenceSnapshot,
};
use crate::publication::CompositeComponentIntent;

/// Product-head observation admitted for the next phase. The observation is
/// carried unchanged; a caller cannot swap a basis between planning steps.
#[derive(Debug)]
pub struct ResolvedExpectedProductHead {
    intent: CompositeComponentIntent,
    expected: ProductBranchObservation,
}

impl ResolvedExpectedProductHead {
    /// Admit an expected observation only when the branch-local reference
    /// cell still selects the complete same image. The observation is carried
    /// unchanged; this function never re-observes a component or asks for an
    /// ambient latest basis.
    pub(crate) fn from_current(
        intent: CompositeComponentIntent,
        expected: ProductBranchObservation,
        current: &ProductBranchReferenceSnapshot,
    ) -> Result<Self, ProductBranchObservationMismatch> {
        if let Some(mismatch) = expected.mismatch_against_snapshot(current) {
            return Err(mismatch);
        }
        Ok(Self { intent, expected })
    }

    pub fn expected(&self) -> &ProductBranchObservation {
        &self.expected
    }

    pub fn intent(&self) -> &CompositeComponentIntent {
        &self.intent
    }
}
