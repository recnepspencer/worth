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
    pub fn source_for_test(&self) -> &super::super::projection_read_origin::ProjectionReadOrigin {
        Self::source(self)
    }
}

impl StructuralWalkObservation {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &StructuralWalkKey {
        Self::key(self)
    }
}
