//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage;

impl WorthQueryApplicationCommitDenial {
    #[doc(hidden)]
    pub fn provider_rejected_with_detail_for_test(
        stage: WorthQueryApplicationCommitDenialStage,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::provider_rejected_with_detail(stage, detail)
    }
}
