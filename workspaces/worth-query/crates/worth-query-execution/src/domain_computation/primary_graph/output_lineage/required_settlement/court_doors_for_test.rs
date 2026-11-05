//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn resolve_required_settlement_for_test(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        completion: &ReadyCompletion,
    ) -> Result<
        Result<Option<AcceptedCurrentCandidate>, FullVerificationReason>,
        RequiredSettlementStop,
    > {
        Self::resolve_required_settlement(self, runtime_authority, schema, completion)
    }
}

impl AcceptedCurrentCandidate {
    #[doc(hidden)]
    pub fn new_for_test(selected: RetainedInputCutoffCandidate, ready: ReadyCompletion) -> Self {
        Self { selected, ready }
    }
}
