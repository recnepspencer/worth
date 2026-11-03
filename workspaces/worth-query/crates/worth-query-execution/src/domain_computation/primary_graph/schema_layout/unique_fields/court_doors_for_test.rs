//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryUniqueFieldFixture {
    #[doc(hidden)]
    pub fn new_for_test( kind: KindId, locator: AspectFieldLocator, equality_index_id: Option<DerivedIndexId>) -> Self {
        Self::new(kind, locator, equality_index_id)
    }
}
