//! One actual applied before/after pair and its original preparation request.
use crate::authority::mutation::MutationApplyOutcome;
use crate::validation::candidate_source::{
    NativeManagedSourceInterpretation, SourcePreparationGroup,
};
use crate::validation::custom_registry::FrozenCustomInvariantRegistry;
use crate::validation::engine::state_view::InvariantStateView;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::selected_read::SelectedRelationalBranchRead;
use worth_relational_contracts::internal_port::history::data::canonical_record_allocation::CanonicalRecordAllocation;
use worth_relational_contracts::internal_port::transactions::data::primitives::RecordRef;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
pub struct SourcePreparationContext<'a> {
    pub state: &'a WorkingState,
    pub outcome: &'a MutationApplyOutcome,
    pub allocations: &'a [CanonicalRecordAllocation],
    pub interpretation: &'a Arc<NativeManagedSourceInterpretation>,
    pub registry: &'a FrozenCustomInvariantRegistry,
    pub home_target: bool,
    pub execution: worth_execution::ExecutionRequest<'a, 'a>,
    pub selected: SelectedRelationalBranchRead<'a>,
    pub proposal:
        &'a crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    pub records: &'a [RecordRef],
}
impl SourcePreparationContext<'_> {
    pub fn affected_groups(&self) -> Vec<&SourcePreparationGroup> {
        if self.home_target {
            return self.registry.all_source_groups().iter().collect();
        }
        let before = InvariantStateView::new(&self.selected, self.selected.version_id());
        let overlay = OverlayStateView::new(&self.selected, self.state);
        let after = InvariantStateView::new(&overlay, self.proposal.proposed_version_id());
        let mut kinds = Vec::new();
        for record in self.records {
            let RecordRef::Entity(id) = record else {
                continue;
            };
            for view in [&before, &after] {
                if let Some(metadata) = view.entity_metadata(*id) {
                    kinds.push(metadata.kind_id);
                }
            }
        }
        self.registry.source_groups(
            kinds.into_iter(),
            self.outcome
                .effect
                .adjacency
                .deltas
                .iter()
                .map(|delta| delta.kind_id),
        )
    }
}
