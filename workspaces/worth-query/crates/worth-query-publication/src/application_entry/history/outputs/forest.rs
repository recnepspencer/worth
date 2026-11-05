//! A runtime-length collection preserves each original typed SDK tree owner.
use super::{retained::*, *};
use std::sync::Arc;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryHistoryOutputNavigationAdvance as Step, WorthQueryRetainedHistoryOutputTree,
};
mod binding;
mod driver;
mod funding;
mod progression;

/// Graph-document output custody for multiple original source demands and roots.
/// Each entry retains its typed program, continuation, slots and disposal owner.
pub struct WorthQueryHistoryWithOutputTrees<S: ApplicationSchema> {
    profile: WorthQueryHistoryOutputProfile,
    trees: Vec<TreeEntry<S>>,
}

struct TreeEntry<S: ApplicationSchema> {
    tree: Box<dyn OriginalTree<S>>,
    // Funds the concrete box and this insertion's collection-capacity growth.
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Type erasure is confined to SDK continuation mechanics. Query still issues
/// every per-tree frame from the immutable registered profile collection.
trait OriginalTree<S: ApplicationSchema> {
    fn profile(&self) -> &WorthQueryHistoryOutputProfile;
    fn runtime(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>;
    fn progress(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: [u8; 32],
        step: Step<'_, '_>,
    ) -> Result<Option<Arc<WorthQueryRetainedHistoryOutputTree>>, WorthQueryHistoryOutputDenial>;
    fn release(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
    fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &[u8; 32],
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
    fn selected(&self, phase: &WorthQueryAdvancementPhase<'_>, key: &[u8; 32]);
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
}
