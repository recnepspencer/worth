use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::InboundWorld;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

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
