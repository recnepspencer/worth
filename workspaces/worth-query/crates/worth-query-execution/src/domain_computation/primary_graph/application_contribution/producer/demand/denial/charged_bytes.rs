//! Retained diagnostic ownership determines the charged-byte contract.
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
impl worth_execution::ChargedBytes for WorthQueryOutputDemandDenial {
    fn additional_charged_bytes(&self) -> u64 {
        use WorthQueryOutputDemandDenialKind as Kind;
        if self.evidence.is_some()
            || self.cleanup.is_some()
            || matches!(
                self.kind.as_ref(),
                Kind::ManagedCleanupRequired(_)
                    | Kind::ProductDelivery(_)
                    | Kind::CorrespondenceDelivery(_)
                    | Kind::BridgeConditional(_)
            )
        {
            return u64::MAX;
        }
        let diagnostic = match self.kind.as_ref() {
            Kind::DiscoveryConnection(cause) => (std::mem::size_of_val(cause.as_ref()) as u64)
                .saturating_add(cause.owned_allocation_capacity_bytes() as u64),
            Kind::UnobservedWrite(cause) => {
                cause.locator().owned_allocation_capacity_bytes() as u64
            }
            _ => 0,
        };
        let text = match &self.subject {
            std::borrow::Cow::Borrowed(_) => 0,
            std::borrow::Cow::Owned(text) => text.capacity() as u64,
        };
        (std::mem::size_of_val(self.kind.as_ref()) as u64)
            .saturating_add(text)
            .saturating_add(diagnostic)
            .saturating_add(self.producer_denial_value_charged_bytes())
    }
}
