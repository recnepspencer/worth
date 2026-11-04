//! Sealed test doors over this module's private members.
use super::*;

impl ComputationCallSequence {
    #[doc(hidden)]
    pub fn edited_for_test<'a>(
        self: &Arc<Self>,
        transition: &CallShapeTransition,
        changes: impl Iterator<Item = (ComputationRead, &'a ComputationCallCharge, Option<u64>)>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial> {
        Self::edited(self, transition, changes, execution)
    }
}
