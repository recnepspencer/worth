//! Original managed consistency refusal retained by the current-output owner.
use super::{WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind};
use crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Cause;
impl WorthQueryCurrentOutputDenial {
    pub(super) fn managed_index(cause: Cause, subject: &str) -> Self {
        let mut denial = Self::new(
            WorthQueryCurrentOutputDenialKind::ManagedIndexSourceDenied,
            subject,
        );
        denial.managed_index = Some(cause);
        denial
    }
    pub fn managed_index_denial(&self) -> Option<&Cause> {
        self.managed_index.as_ref()
    }
}
