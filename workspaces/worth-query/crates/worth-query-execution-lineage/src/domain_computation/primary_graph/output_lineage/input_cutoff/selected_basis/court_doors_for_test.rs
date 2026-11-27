//! Sealed test doors over this module's private members.
use super::*;

impl<'snapshot> PreparedInputCutoffBasis<'snapshot> {
    #[doc(hidden)]
    pub fn prepare_for_test(
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'snapshot>,
    ) -> Result<Self, InputCutoffVerificationStop> {
        Self::prepare(runtime, basis)
    }
}
