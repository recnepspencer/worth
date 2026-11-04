//! Sealed test doors over this module's private members.
use super::*;

impl WorthQuerySourceExpectationDenial {
    #[doc(hidden)]
    pub fn source_retention_denied_for_test(
        subject: &str,
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
    ) -> Self {
        Self::source_retention_denied(subject, denial)
    }
}
