//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductWorldResources {
    #[doc(hidden)]
    pub fn budgets_for_test(&self) -> &RuntimeWorldBudgets {
        Self::budgets(self)
    }
}

impl WorthQueryProductWorldResources {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        RuntimeWorldBudgets,
        WorthQueryProductWorldClock,
        super::super::super::WorthQueryInvalidationResources,
        WorthQueryProductExecution,
    ) {
        Self::into_parts(self)
    }
}

impl WorthQueryProductExecution {
    #[doc(hidden)]
    pub fn authority_for_test(&self) -> &Option<Arc<ExecutionAuthority>> {
        &self.authority
    }
}

impl WorthQueryProductExecution {
    #[doc(hidden)]
    pub fn policy_for_test(&self) -> &ExecutionRequestPolicy {
        &self.policy
    }
}
