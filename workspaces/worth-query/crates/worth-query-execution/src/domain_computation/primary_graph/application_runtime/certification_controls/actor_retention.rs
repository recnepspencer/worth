//! Runs the existing native retention owner, without changing policy or pins.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn run_actor_retention_pass_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        (usize, usize),
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        let execution = phase.execution_request_for(&self.product_runtime).map_err(|_| {
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase,
                "actor retention",
            )
        })?;
        let handle = self
            .runtime
            .primary_graph()
            .expect("installed primary graph")
            .integration_handle();
        handle.with_query_runtime_mut(&execution, |runtime, _| {
            runtime.retention().run_pass(&execution)
                .map(|actual| (actual.entity_reclaimed, actual.relation_reclaimed))
                .map_err(|cause| crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "actor retention"))
        })
    }
}
