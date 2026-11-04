//! Sealed test doors over this module's private members.
use super::*;

impl RetirementMetadata {
    #[doc(hidden)]
    pub fn created_at_for_test(&self) -> &VersionId {
        &self.created_at
    }
}

impl RetirementMetadata {
    #[doc(hidden)]
    pub fn deleted_at_for_test(&self) -> &VersionId {
        &self.deleted_at
    }
}

impl RetirementMetadata {
    #[doc(hidden)]
    pub fn kind_for_test(&self) -> &KindId {
        &self.kind
    }
}

impl RetirementMetadata {
    #[doc(hidden)]
    pub fn created_at_mut_for_test(&mut self) -> &mut VersionId {
        &mut self.created_at
    }
}
