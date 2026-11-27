//! Public #358: a transported consumer is readmitted with its whole consumed
//! chain. After `is_current` held at the head, the chain is re-identified at
//! the consumer's own coordinate, each re-identified row's publication must
//! still be owned there, and the chain is posted bottom-up at that head
//! before the re-rooted evidence is installed. Any refusal readmits nothing;
//! the demand then executes as it would without the transported row.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::current_output::retained_candidates::WorthQueryRetainedOutputCandidate;
use crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop;
use crate::domain_computation::primary_graph::output_reuse::publication_owned_at;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::RelationalSourceOwnerInternalPort as _;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The head the chain was posted at, or `None` when readmission refuses.
    pub(super) fn readmit_consumed_chain(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>,
        candidate: &WorthQueryRetainedOutputCandidate,
    ) -> Result<Option<PositionedRelationalSnapshot>, WorthQueryOutputDemandDenial> {
        let graph = &self.runtime.primary_provider.graph_ref();
        let snapshot = selected.application_basis().snapshot_handle();
        let Ok(head) =
            graph.with_runtime(execution, |native| native.read_truth().positioned_snapshot(snapshot))
        else {
            return Ok(None);
        };
        let Some(plan) = graph
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            // Called only after `restored_output_is_current` compared the
            // whole chain's facts in full at this head (public #429).
            .reroot_consumed(candidate, &head, true)
        else {
            return Ok(None);
        };
        for publication in &plan.publications {
            if !publication_owned_at(execution, publication, selected)? {
                return Ok(None);
            }
        }
        let source_owner = graph.source_owner_ref();
        match source_owner.mint_mark_cell_at_head(execution, head.branch_id()) {
            Ok(()) => {}
            // As every readmission marks: a head cell that cannot be minted
            // leaves the output unmarked, so here it is not readmitted.
            Err(HeadCellRegistrationStop::Admission(_) | HeadCellRegistrationStop::LookupChanged)
            | Err(HeadCellRegistrationStop::Native(_)) => return Ok(None),
        }
        let posted = graph.with_runtime(execution, |native| {
            source_owner
                .invalidation_owner
                .post_readmitted_chain(native, snapshot, &head, &plan.nodes)
        });
        // A stop leaves rows proven at `head` requiring verification, as a
        // stopped mark does on every other readmission.
        if !matches!(posted, Ok(true)) {
            return Ok(None);
        }
        plan.install();
        Ok(Some(head))
    }
}
