//! Owner handoff of a performed inbound completion to conditional delivery.

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::WorthQueryInboundTerminalOwnerResult;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The completion creates a new Query-owned entity. No installed exact
    /// record route can name that entity before this commit; whole-graph and
    /// bootstrap routes receive the actual commit through the bounded journal.
    /// The journal handoff precedes disarming the one-shot direct witness.
    pub(in crate::domain_computation) fn settle_inbound_conditional_delivery(
        &self,
        terminal: &WorthQueryInboundTerminalOwnerResult,
    ) -> bool {
        terminal.settle_after_conditional_handoff(|| {
            let relational = terminal
                .publication()
                .publication()
                .component_results()
                .relational_commit_result()
                .expect("performed inbound Relational completion retains its commit");
            self.primary_provider
                .record_conditional_commit(&relational.commit, std::iter::empty());
        })
    }
}
