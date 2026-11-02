//! Retained managed consistency refusals preserve their original typed cause.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial as Cause;
impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn managed_index(
        cause: Cause,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        let mut denial = Self::new(
            WorthQueryOutputDemandDenialKind::ManagedIndexSourceDenied,
            subject,
        );
        denial.evidence_mut().managed_index = Some(cause);
        denial
    }
    pub fn managed_index_denial(&self) -> Option<&Cause> {
        self.evidence.as_ref()?.managed_index.as_ref()
    }
}
