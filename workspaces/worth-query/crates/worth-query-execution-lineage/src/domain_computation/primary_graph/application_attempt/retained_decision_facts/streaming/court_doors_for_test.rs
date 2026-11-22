//! Sealed test doors over this module's private members.
use super::*;

impl AdmittedFactKey {
    #[doc(hidden)]
    pub fn from_borrowed_for_test(
        locator: impl Fn(&mut dyn fmt::Write) -> fmt::Result,
        predicate: Option<(&AspectValue, usize)>,
        policy: super::super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::from_borrowed(locator, predicate, policy)
    }
}
