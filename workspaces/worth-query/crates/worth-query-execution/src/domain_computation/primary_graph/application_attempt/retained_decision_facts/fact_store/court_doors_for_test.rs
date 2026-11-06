//! Sealed test doors over this module's private members.
use super::*;

impl AuthoringSourceFacts {
    #[doc(hidden)]
    pub fn capture_for_test(
        &mut self,
        fact: Fact,
        control: StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        Self::capture(self, fact, control)
    }
}

impl AuthoringSourceFacts {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        control: StorageControl<'_, '_>,
    ) -> Result<RetainedSourceFacts, StoreDenial> {
        Self::finish(self, control)
    }
}

impl RetainedSourceFacts {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}
