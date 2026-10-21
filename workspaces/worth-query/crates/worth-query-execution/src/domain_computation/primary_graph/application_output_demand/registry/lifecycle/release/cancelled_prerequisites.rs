//! Cancellation Drop receives its cleanup payer only from the cleanup owner.
use super::super::super::PreparedPrerequisiteClaims;
impl Drop for PreparedPrerequisiteClaims {
    fn drop(&mut self) {
        self.cancel_unpublished_prerequisites();
    }
}
