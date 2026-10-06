use super::{UiExpressionOutcome, UiExpressionRuntimeState};

impl UiExpressionRuntimeState {
    /// Retains `outcome` for the expression named `identity` as if its last
    /// settle had produced it, advancing its revision as a changed outcome
    /// does. Its operand facts stay those it read. It stands in for a Query
    /// revalidation world when a test needs a retained-stale posture. The
    /// expression must be a leaf: no reader is re-settled for the new
    /// revision, so a reader would keep reading the prior one.
    pub(crate) fn retain_outcome(&mut self, identity: &str, outcome: UiExpressionOutcome) {
        let slot = self
            .catalog
            .slot_of(identity)
            .expect("the expression is installed");
        assert!(
            self.catalog.dependencies().dependents_of(slot).is_empty(),
            "`{identity}` has readers, which a retained outcome would not re-settle"
        );
        let mut record = self
            .records
            .get(slot)
            .cloned()
            .expect("the expression is settled");
        record.outcome = outcome;
        record.outcome_revision = record.outcome_revision.saturating_add(1);
        assert!(self.records.admit(record).is_ok());
    }
}
