pub use super::WorthQueryApplicationEffectProgram;
pub use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;

impl<Schema, Operation, Input, Scope>
    crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
        Schema,
        Operation,
        Input,
        Scope,
    >
{
    #[inline]
    pub(crate) fn validate_current_authority(
        &self,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        self.read_set.admission.validate_current_authority()
    }

    #[doc(hidden)]
    pub fn validate_current_authority_for_test(
        &self,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        Self::validate_current_authority(self)
    }
}
