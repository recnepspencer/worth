//! Lifecycle records use the one serialized Query publisher.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableCompositeCommitIdentity;

impl QueryObligationHistory {
    /// Same canonical source-root meaning and scope; liveness belongs to the
    /// source registry, which decides whether each candidate may be retired.
    pub(in crate::domain_computation::primary_graph) fn discovered_source_predecessors(
        &self,
        current: DurableCompositeCommitIdentity,
    ) -> Vec<DurableCompositeCommitIdentity> {
        let state = self.state();
        let Some(output) = state.journal.as_ref().map(|journal| &journal.ledger.output) else {
            return Vec::new();
        };
        let Some(current_row) = output.sources.get(&current) else {
            return Vec::new();
        };
        output
            .sources
            .values()
            .filter(|prior| {
                prior.discovery_recipe.is_some()
                    && prior.commit.ordinal() < current.ordinal()
                    && prior.commit.owner() == current.owner()
                    && prior.revision == current_row.revision
                    && prior.root == current_row.root
                    && prior.branch == current_row.branch
                    && prior.lifecycle == current_row.lifecycle
                    && prior.scope == current_row.scope
                    && !output.demands.values().any(|demand| {
                        !output.completed.contains(&demand.acquire_position)
                            && demand.performed_sources.contains(&prior.commit)
                    })
            })
            .map(|prior| prior.commit)
            .collect()
    }
    pub(in crate::domain_computation::primary_graph) fn output_sources(
        &self,
    ) -> Vec<QueryOutputSourceObligation> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| journal.ledger.output.sources.values().cloned().collect())
            .unwrap_or_default()
    }
    pub(in crate::domain_computation::primary_graph) fn retain_output_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
        row: QueryOutputSourceObligation,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if let Some(prior) = self
            .state()
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.output.sources.get(&row.commit))
        {
            return if prior == &row {
                Ok(())
            } else {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputSourceRecord::Retain(row),
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn discharge_output_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
        commit: DurableCompositeCommitIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if !self
            .state()
            .journal
            .as_ref()
            .is_some_and(|journal| journal.ledger.output.sources.contains_key(&commit))
        {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputSourceRecord::Discharge { commit },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn bind_output_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        commit: DurableCompositeCommitIdentity,
        mut sources: Vec<([u8; 32], (u32, u64, u32))>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        sources.sort_unstable();
        sources.dedup();
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), execution)?;
        let prior = self
            .state()
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.output.sources.get(&commit).cloned())
            .ok_or(QueryObligationDenial::UnknownObligation)?;
        if let Some(known) = prior.bindings {
            return if known == sources {
                Ok(())
            } else {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryOutputSourceRecord::Bind { commit, sources },
            execution,
        )
    }
}
