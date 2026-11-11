//! Cold canonical completion-source readmission before Home service.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_inbound_terminal_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        unpublished_native: &std::collections::BTreeSet<
            worth_relational::facade::history::CommitId,
        >,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial,
    > {
        use crate::domain_computation::primary_graph::provider::inbound_terminal_index::InboundTerminalSource;
        let originals = self
            .runtime
            .primary_provider
            .readmit_inbound_completion_sources(
                execution,
                &self.runtime.product_runtime.owner.observation_port(),
                unpublished_native,
            )
            .map_err(super::super::rebuild::completion_read_denial)?;
        for (world, entity) in originals {
            let source = InboundTerminalSource::Readmitted { world, entity };
            let unbounded =
                std::num::NonZeroUsize::new(usize::MAX).expect("cold request controls traversal");
            let terminal =
                self.rebuild_terminal_from_source(execution, &source, unbounded, unbounded)?;
            let InboundTerminalSource::Readmitted { world, entity } = source else {
                unreachable!("actual readmitted source")
            };
            self.runtime
                .primary_provider
                .retain_readmitted_inbound_terminal(terminal, world, entity)?;
        }
        Ok(())
    }
}
