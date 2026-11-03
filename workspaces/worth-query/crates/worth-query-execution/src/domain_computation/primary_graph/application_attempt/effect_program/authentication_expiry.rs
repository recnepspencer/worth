//! An effect program may expire only the authentication of its own admission.
use super::WorthQueryApplicationEffectProgram;
impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn expire_authentication_for_test(&mut self) {
        self.read_set.admission.expire_authentication_for_test();
    }
}
