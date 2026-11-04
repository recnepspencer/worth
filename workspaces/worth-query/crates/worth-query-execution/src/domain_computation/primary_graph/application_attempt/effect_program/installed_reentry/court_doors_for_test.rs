//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn validate_current_authority_for_test(
        &self,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::validate_current_authority(self)
    }
}
