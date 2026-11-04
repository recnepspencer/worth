//! Sealed test doors over this module's private members.
use super::*;

impl StructuralWalkObservation {
    #[doc(hidden)]
    pub fn native_for_test(&self) -> &PinnedStructuralWalk {
        Self::native(self)
    }
}

impl StructuralWalkObservation {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &SnapshotHandle {
        Self::source(self)
    }
}
