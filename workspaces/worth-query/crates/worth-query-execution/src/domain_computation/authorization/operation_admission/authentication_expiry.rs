//! Attenuate the genuine receiver's monotonic authentication lifetime without sleeping.
use super::WorthQueryAdmittedApplicationOperation;
impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn expire_authentication_for_test(&mut self) {
        self.core.authentication_valid_until = std::time::Instant::now();
    }
}
