//! Sealed test doors over this module's private members.
use super::*;

impl<'scope, 'authority> StorageControl<'scope, 'authority> {
    #[doc(hidden)]
    pub fn new_for_test( policy: ExecutionAllocationPolicy<'scope, 'authority>, request: Option<&'scope WorthQueryRequestScope>) -> Self {
        Self::new(policy, request)
    }
}
