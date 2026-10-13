//! Cancellation Drop receives its cleanup payer only from the cleanup owner.
use super::super::super::PreparedPrerequisiteClaims;
use super::CleanupWalk;
impl Drop for PreparedPrerequisiteClaims {
    fn drop(&mut self) {
        let mut cleanup = CleanupWalk { _release: () };
        self.cancel_unpublished_prerequisites(&mut cleanup);
    }
}
