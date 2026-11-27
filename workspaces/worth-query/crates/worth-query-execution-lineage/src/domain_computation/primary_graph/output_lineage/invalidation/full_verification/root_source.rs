//! Retained lineage source evidence may outlive a failed index registration.

use super::*;

impl FullVerificationImage<'_> {
    pub fn root_source_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EvidenceView<'_>,
    ) -> Result<bool, FullVerificationStop> {
        if !observed_on_selected(self.selected, source.selected_native_root())? {
            return Ok(false);
        }
        let Some(facts) = source.source_facts() else {
            return Ok(false);
        };
        let acceptance = super::super::super::RestoredAcceptance::new(
            execution,
            self.runtime,
            self.basis,
            source.historical,
        );
        for fact in facts {
            if !(fact_is_current(execution, fact, self.runtime, self.basis)?
                || acceptance.moved_fact_holds(fact)?)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn require_complete_root_upstream(
        &self,
        source: EvidenceView<'_>,
        terminal: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(), FullVerificationStop> {
        let mut actual = OrdSet::new();

        for upstream in source.upstream() {
            actual.insert(Arc::clone(upstream.identity()));
        }

        let row = self
            .state
            .settlements
            .get(terminal)
            .ok_or(FullVerificationStop::OutputEvidenceUnavailable)?;

        if actual.len() != row.consumed_upstream.len() {
            return Err(FullVerificationStop::OutputEvidenceUnavailable);
        }
        for upstream in &row.consumed_upstream {
            if !actual.contains(upstream) {
                return Err(FullVerificationStop::OutputEvidenceUnavailable);
            }
        }
        Ok(())
    }
}

mod court_doors_for_test;
