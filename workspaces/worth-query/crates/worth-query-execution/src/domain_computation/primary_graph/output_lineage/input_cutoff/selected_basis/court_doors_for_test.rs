//! Sealed test doors over this module's private members.
use super::*;

impl<'snapshot> PreparedInputCutoffBasis<'snapshot> {
    #[doc(hidden)]
    pub fn prepare_for_test(
        runtime: &RelationalRuntime,
        snapshot: &'snapshot SnapshotHandle,
    ) -> Result<Self, InputCutoffVerificationStop> {
        Self::prepare(runtime, snapshot)
    }
}
