//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationAttemptOperationControl {
    #[doc(hidden)]
    pub fn caller_pass_for_test(&self) -> &Arc<caller_pass::CallerPassControl> {
        &self.caller_pass
    }
}
