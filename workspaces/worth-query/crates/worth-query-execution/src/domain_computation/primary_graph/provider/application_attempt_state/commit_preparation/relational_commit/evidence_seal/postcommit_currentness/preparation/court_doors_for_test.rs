//! Sealed test doors over this module's private members.
use super::*;

impl PreparedSourceFactRebase {
    #[doc(hidden)]
    pub fn admit_for_test(
        facts: Vec<WorthQueryApplicationObservedFact>,
        moved: Arc<[usize]>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        Self::admit(facts, moved, control)
    }
}
