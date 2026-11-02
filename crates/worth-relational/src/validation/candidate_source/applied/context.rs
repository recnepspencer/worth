//! One actual applied before/after pair and its original preparation request.
use super::*;
pub(in crate::validation::candidate_source) struct SourcePreparationContext<'a> {
    pub draft: &'a AppliedMutationDraft,
    pub execution: worth_execution::ExecutionRequest<'a, 'a>,
    pub runtime: &'a RelationalPreparationRuntime,
    pub selected: SelectedRelationalBranchRead<'a>,
    pub proposal: &'a crate::mvcc::RelationalMutationProposalIdentity,
}
impl SourcePreparationContext<'_> {
    pub(super) fn affected_groups(&self) -> Vec<&SourcePreparationGroup> {
        let before = InvariantStateView::new(&self.selected, self.selected.version_id());
        let overlay = OverlayStateView::new(&self.selected, self.draft.state());
        let after = InvariantStateView::new(&overlay, self.proposal.proposed_version_id());
        let mut kinds = Vec::new();
        for record in &self.draft.outcome.effect.publication.changed_records {
            let RecordRef::Entity(id) = record else {
                continue;
            };
            for view in [&before, &after] {
                if let Some(metadata) = view.entity_metadata(*id) {
                    kinds.push(metadata.kind_id);
                }
            }
        }
        self.runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .source_groups(
                kinds.into_iter(),
                self.draft
                    .outcome
                    .effect
                    .adjacency
                    .deltas
                    .iter()
                    .map(|delta| delta.kind_id),
            )
    }
}
