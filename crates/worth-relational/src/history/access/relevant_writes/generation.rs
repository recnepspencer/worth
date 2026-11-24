//! Discardable posting and prefix custody for one canonical branch selection.
use super::{posting_key::AuthoredWritePostingKey as Key, RelationalAuthoredWriteDenial as Denial};
use crate::history::data::CommitId;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionOrderedMap, ExecutionRequest};

#[derive(Clone, Debug, Default)]
pub(crate) struct AuthoredWriteGeneration {
    pub(crate) writes: ExecutionOrderedMap<Key, Postings>,
    pub(crate) prefix: ExecutionOrderedMap<CommitId, u64>,
    pub(crate) managed_last_gap: Option<u64>,
    pub(crate) managed_run_start: u64,
    pub(crate) managed_run_catalog: Option<
        Arc<crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog>,
    >,
    pub(crate) incomplete_indexes: ExecutionOrderedMap<u64, ()>,
    pub(crate) incomplete_adjacency: ExecutionOrderedMap<u64, AdjacencyHistoryGap>,
    pub(crate) index_coverage: ExecutionOrderedMap<
        u64,
        crate::runtime::state::subsystems::indexing::catalog_coverage::IndexCatalogCoverage,
    >,
    pub(crate) index_run_start: u64,
    pub(crate) index_run_last:
        Option<crate::runtime::state::subsystems::indexing::catalog_coverage::IndexCatalogCoverage>,
    pub(crate) ordinal: u64,
    pub(crate) unsupported_merge: bool,
    _funding: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

/// Context belongs to the same discardable generation as the incomplete ordinal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AdjacencyHistoryGap {
    pub(crate) commit: CommitId,
    pub(crate) relation: Option<crate::identity::data::RelationId>,
    pub(crate) kind: Option<crate::identity::data::KindId>,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Postings(pub(crate) ExecutionOrderedMap<u64, CommitId>);
impl ChargedBytes for Postings {
    // The shared cells retain their actual original allocation grants.
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl AuthoredWriteGeneration {
    pub(crate) fn unsupported_merge() -> Self {
        Self {
            unsupported_merge: true,
            ..Self::default()
        }
    }

    pub(crate) fn advance(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        commit: CommitId,
    ) -> Result<(), Denial> {
        self._funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<AuthoredWriteGeneration>()
                    + std::mem::size_of::<worth_execution::ExecutionMemoryReservation>()
                    + 2 * std::mem::size_of::<usize>()) as u64,
            )?
            .map(Arc::new);
        self.ordinal = self.ordinal.checked_add(1).ok_or(Denial::SizeOverflow)?;
        super::ordered_visit(context, self.prefix.len())?;
        self.prefix
            .insert_cloned_native_for_request(&commit, &self.ordinal, execution)?;
        Ok(())
    }

    pub(crate) fn append(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        key: &Key,
        commit: CommitId,
    ) -> Result<(), Denial> {
        super::ordered_visit(context, self.writes.len())?;
        let mut postings = self.writes.get(key).cloned().unwrap_or_default();
        super::ordered_visit(context, postings.0.len())?;
        postings
            .0
            .insert_cloned_native_for_request(&self.ordinal, &commit, execution)?;
        super::ordered_visit(context, self.writes.len())?;
        self.writes
            .insert_cloned_for_request(key, &postings, execution)?;
        Ok(())
    }
}
