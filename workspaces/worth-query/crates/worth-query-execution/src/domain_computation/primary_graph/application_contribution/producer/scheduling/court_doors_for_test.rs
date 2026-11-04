//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInstalledOutputProducerRoute {
    #[doc(hidden)]
    pub fn new_for_test(
        scheduling: Arc<BridgeInstalledConditionalLowering>,
        output_comparison: Option<super::super::output_evaluation::InstalledOutputComparison>,
    ) -> Self {
        Self {
            scheduling,
            output_comparison,
        }
    }
}
