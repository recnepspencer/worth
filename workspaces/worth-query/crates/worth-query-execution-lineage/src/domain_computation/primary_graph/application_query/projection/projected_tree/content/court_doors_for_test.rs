//! Sealed test doors over this module's private members.
use super::*;

impl<'a, E> ContentWriter<'a, E> {
    #[doc(hidden)]
    pub fn new_for_test(
        admit: &'a mut dyn FnMut(u64) -> Result<(), E>,
        overflow: &'a dyn Fn() -> E,
    ) -> Self {
        Self::new(admit, overflow)
    }
}
