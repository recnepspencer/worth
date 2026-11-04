//! Diagnostic formatting for typed output-demand refusals.
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;

impl WorthQueryOutputDemandDenial {
    /// Names the receiving boundary without replacing original kind or custody.
    pub(in crate::domain_computation::primary_graph) fn with_subject(
        mut self,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        self.subject = subject.into();
        self
    }
}

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
