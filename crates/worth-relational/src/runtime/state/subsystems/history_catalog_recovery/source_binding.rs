//! Bind each reconstructed current root before its original source becomes history.
use super::{HistorySubsystem, RelationalHistoryRecoveryDenial};
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use std::sync::Arc;

impl HistorySubsystem {
    /// Reconstructive single-commit update. The catalog's existing full-payload
    /// classifier alone decides whether replay-issued private evidence survives.
    pub(crate) fn synchronize_recovered_source_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &PositionedCanonicalCommit,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<CanonicalCommitEnvelope>, RelationalHistoryRecoveryDenial> {
        let (canonical, residency, roots) = self.with_ledger_mut(|ledger| {
            ledger
                .commit_catalog
                .synchronize_recovered_envelope(source)
                .map_err(|denial| format!("recovered source synchronization denied: {denial:?}"))?;
            let artifact = ledger
                .commit_catalog
                .get(source.commit.commit_id)
                .ok_or("recovered source has no catalog artifact")?;
            let canonical = artifact
                .resident_envelope()
                .ok_or("recovered source has no admitted canonical body")?;
            let residency = artifact.envelope_residency();
            ledger
                .commit_envelopes
                .insert(source.commit.commit_id, residency.clone());
            ledger
                .patch_stream_index
                .insert(source.position(), source.commit.commit_id);
            Ok::<_, String>((canonical, residency, artifact.roots().clone()))
        })?;
        let mut cell = self
            .branch_cell(&source.branch_context)
            .ok_or_else(|| "recovered source branch is unavailable".to_owned())?;
        let Some(root) = cell.root() else {
            return Ok(canonical);
        };
        if root.commit_id() != Some(source.commit.commit_id) {
            return Ok(canonical);
        }
        self.canonical_publication_routes
            .rebind_recovered_latest(source, residency, roots)
            .map_err(str::to_owned)?;
        if root.links_envelope(&canonical) {
            return Ok(canonical);
        }
        let next = if Arc::ptr_eq(&canonical, source.canonical_arc()) {
            root.readmit_canonical_envelope(execution, source, symbols)
        } else {
            root.relink_canonical_envelope(execution, canonical.clone(), symbols)
        }
        .map_err(RelationalHistoryRecoveryDenial::Root)?;
        let retirement = self
            .reserve_branch_head_retirement(cell.identity(), &root, cell.head_retention())
            .map_err(|denial| format!("recovered source root retention denied: {denial:?}"))?;
        cell.install_root(next.clone());
        retirement.transfer_head(&next).replace_head();
        Ok(canonical)
    }
}
