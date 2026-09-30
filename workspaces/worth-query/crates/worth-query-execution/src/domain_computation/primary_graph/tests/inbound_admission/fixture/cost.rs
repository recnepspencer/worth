use super::InboundWorld;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

impl InboundWorld {
    pub fn attempt_dispatch_with_scope(
        &self,
        seed: u8,
        text: &str,
        request: &WorthQueryRequestScope,
    ) -> WorthQueryApplicationCommitOutcome {
        self.attempt_operation_on(
            self.application.current_world(),
            seed,
            text,
            true,
            Some(request),
        )
    }
}
