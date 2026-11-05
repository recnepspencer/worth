//! Sealed test doors over this module's private members.
use super::*;

impl PreparedRequiredInputs {
    #[doc(hidden)]
    pub fn new_for_test(runtime: u64, root: u64, owner: WorthQueryOutputDemandRegistry) -> Self {
        Self {
            runtime,
            root,
            owner,
        }
    }
}
