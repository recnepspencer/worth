//! Law 2, scope: verification consults its own runtime's installed resources.
use super::SourceInvalidationOwner;
impl SourceInvalidationOwner {
    pub(in crate::domain_computation) fn invalidation_equivalence_is_armed(&self) -> bool {
        self.resources.invalidation_equivalence_is_armed()
    }
    pub(in crate::domain_computation) fn record_invalidation_shadow_check(&self) {
        self.resources.record_invalidation_shadow_check();
    }
}
