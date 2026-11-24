//! Access for the existing cross-package corruption and custody fixtures.
use super::*;

#[doc(hidden)]
pub trait RecoveryPlanFieldsFixturePort {
    fn tail_log_mut_for_test(
        &mut self,
    ) -> &mut Vec<crate::durability::migration::ReadmittedCanonicalCommit>;
}

impl RecoveryPlanFieldsFixturePort for RecoveryPlan {
    fn tail_log_mut_for_test(
        &mut self,
    ) -> &mut Vec<crate::durability::migration::ReadmittedCanonicalCommit> {
        &mut self.tail_log
    }
}
