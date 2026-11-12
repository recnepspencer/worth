//! Preserve the connection owner's actual typed refusal and identity.
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use worth_query_declaration::facade::application_program::WorthQueryRequiredOutputConnectionDenial;
impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn discovery_connection_denied(
        connection: &'static str,
        cause: WorthQueryRequiredOutputConnectionDenial,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::DiscoveryConnection(Box::new(cause)),
            connection,
        )
    }
    /// The actual installed discovery adapter's refusal, without a formatted replacement.
    pub fn discovery_connection_denial(&self) -> Option<&WorthQueryRequiredOutputConnectionDenial> {
        match self.kind.as_ref() {
            WorthQueryOutputDemandDenialKind::DiscoveryConnection(cause) => Some(cause),
            _ => None,
        }
    }
}
