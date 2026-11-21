//! Sealed test doors over this module's private members.
use super::*;

impl PostingProjection {
    #[doc(hidden)]
    pub fn ordinals_for_test(&self) -> &OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        &self.ordinals
    }
}

impl PostingProjection {
    #[doc(hidden)]
    pub fn id_for_test(&self) -> &u64 {
        &self.id
    }
}
