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
        self.application
            .with_application_advancement(request, |phase| {
                self.attempt_operation_on(
                    &phase,
                    &phase
                        .execution_request_for(self.application.product_runtime())
                        .expect("phase belongs to application"),
                    self.application.current_world(),
                    seed,
                    text,
                    true,
                    Some(request),
                )
            })
            .expect("installed host call opens")
    }
}
