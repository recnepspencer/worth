//! Sealed test doors over this module's private members.
use super::*;
use worth_runtime_bridge::facade::BridgeInstalledConditionalLowering;

impl WorthQueryInstalledOutputProducerRoute {
    #[doc(hidden)]
    pub fn new_for_test(
        scheduling: Arc<BridgeInstalledConditionalLowering>,
        output_comparison: Option<super::super::output_evaluation::InstalledOutputComparison>,
    ) -> Self {
        Self {
            scheduling: InstalledConditionalRoute::from_installation(scheduling),
            output_comparison,
        }
    }
}
