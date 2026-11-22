use super::managed_graph_execution::WorthQueryManagedGraphExecution;
use crate::domain_computation::WorthQueryProviderExecutionReleaseEvidence;

pub(super) struct WorthQueryManagedProviderExecutionRelease {
    evidence: WorthQueryProviderExecutionReleaseEvidence,
    memory: super::super::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemoryArena,
}

impl WorthQueryManagedGraphExecution {
    pub(super) fn release_provider_execution(self) -> WorthQueryManagedProviderExecutionRelease {
        let memory = self.memory;
        let evidence = self.execution.release();
        WorthQueryManagedProviderExecutionRelease { evidence, memory }
    }
}

impl WorthQueryManagedProviderExecutionRelease {
    pub(super) const fn evidence(&self) -> &WorthQueryProviderExecutionReleaseEvidence {
        &self.evidence
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryProviderExecutionReleaseEvidence,
        super::super::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemoryArena,
    ){
        (self.evidence, self.memory)
    }
}
