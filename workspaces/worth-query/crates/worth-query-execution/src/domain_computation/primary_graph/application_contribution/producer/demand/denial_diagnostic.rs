//! Diagnostic formatting for typed output-demand refusals.
use super::WorthQueryOutputDemandDenial;

impl std::fmt::Display for WorthQueryOutputDemandDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "output demand denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOutputDemandDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.managed_index_denial()
            .map(|cause| cause as &dyn std::error::Error)
    }
}
