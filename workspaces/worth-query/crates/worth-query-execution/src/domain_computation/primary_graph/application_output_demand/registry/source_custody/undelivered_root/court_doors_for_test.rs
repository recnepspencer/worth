//! Sealed test doors over this module's private members.
use super::*;

impl SourceCustody {
    #[doc(hidden)]
    pub fn return_undelivered_root_for_test(
        &mut self,
        identity: &SourceEpoch,
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
    ) {
        Self::return_undelivered_root(self, identity, scope)
    }
}
