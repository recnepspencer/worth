//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCapabilityDecisionRuleBindings {
    #[doc(hidden)]
    pub fn allow_for_test(&self) -> &usize {
        &self.allow
    }
}

impl WorthQueryCapabilityDecisionRuleBindings {
    #[doc(hidden)]
    pub fn conflict_for_test(&self) -> &Option<usize> {
        &self.conflict
    }
}

impl WorthQueryCapabilityDecisionRuleBindings {
    #[doc(hidden)]
    pub fn deny_for_test(&self) -> &Option<usize> {
        &self.deny
    }
}

impl WorthQueryCapabilityDecisionRuleBindings {
    #[doc(hidden)]
    pub fn distinct_actor_for_test(&self) -> &Option<usize> {
        &self.distinct_actor
    }
}

impl WorthQueryCapabilityDecisionRuleBindings {
    /// Not the plain field name: the store's physical backend owns that control name.
    #[doc(hidden)]
    pub fn grant_field_for_test(&self) -> &usize {
        &self.grant
    }
}

impl WorthQueryCapabilityPathTemplate {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &[u8; 32] {
        &self.identity
    }
}

impl WorthQueryCapabilityDecisionRuleBindings {
    #[doc(hidden)]
    pub fn separation_of_duty_for_test(&self) -> &Option<usize> {
        &self.separation_of_duty
    }
}

impl WorthQueryCapabilityRuleBinding {
    #[doc(hidden)]
    pub fn path_requirements_for_test(&self) -> &[Vec<usize>] {
        Self::path_requirements(self)
    }
}
