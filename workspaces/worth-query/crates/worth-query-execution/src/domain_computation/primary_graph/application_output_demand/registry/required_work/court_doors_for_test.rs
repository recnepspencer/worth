//! Sealed test doors over this module's private members.
use super::*;

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn new_for_test(key: Arc<WorthQueryOutputDemandKey>, queue: &Arc<RequiredWorkQueue>) -> Self {
        Self::new(key, queue)
    }
}
