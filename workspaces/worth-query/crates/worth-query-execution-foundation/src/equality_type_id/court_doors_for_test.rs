//! Sealed test doors over this module's private members.
use super::*;

impl EqualityTypeId {
    #[doc(hidden)]
    pub fn of_for_test<T: ?Sized + 'static>() -> Self {
        Self::of::<T>()
    }
}
