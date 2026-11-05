//! Sealed test doors over this module's private members.
use super::*;

impl CheckpointPriorSelectionDenial {
    #[doc(hidden)]
    pub fn into_capture_denial_for_test(
        self,
    ) -> worth_relational::facade::durability::DurabilityError {
        Self::into_capture_denial(self)
    }
}
