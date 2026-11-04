//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryUniqueFieldFixture {
    #[doc(hidden)]
    pub fn new_for_test(
        kind: KindId,
        locator: AspectFieldLocator,
        equality_index_id: Option<DerivedIndexId>,
    ) -> Self {
        Self::new(kind, locator, equality_index_id)
    }
}

impl<'layout> WorthQueryUniqueFields<'layout> {
    #[doc(hidden)]
    pub fn none_for_test() -> WorthQueryUniqueFields<'static> {
        Self::none()
    }
}
