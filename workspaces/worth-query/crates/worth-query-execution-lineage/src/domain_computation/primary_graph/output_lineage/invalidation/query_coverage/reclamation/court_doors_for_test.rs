//! Sealed test doors over this module's private members.
use super::*;

impl QueryReadBacking {
    #[doc(hidden)]
    pub fn keys_for_test(&self) -> &OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        Self::keys(self)
    }
}
