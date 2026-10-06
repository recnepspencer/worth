use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};
use crate::visibility::materialization::read_records::reader::adjacency_work_ledger::AdjacencyLeaseLedger;

use super::{
    RelationalAdjacencyDirection, RelationalBorrowedRecordReadDenial, VisibilityProjectionView,
};
use crate::visibility::snapshot_states::SnapshotStateBasis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalAdjacencyVisit {
    Prepare { work: u64, bytes: u64 },
    List,
    Relation(RelationId),
}

impl VisibilityProjectionView<'_> {
    /// Exact selected list revision; the caller admits the anchor metadata and
    /// list visits. No kind names or read records are materialized.
    pub fn exact_adjacency_structural_revision(
        &self,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
    ) -> Result<Option<VersionId>, RelationalBorrowedRecordReadDenial> {
        match self.exact_adjacency_structural_revision_admitted(anchor, kind, direction, |_| {
            Ok::<(), std::convert::Infallible>(())
        }) {
            Ok(result) => result,
            Err(never) => match never {},
        }
    }

    /// Admit each exact-root read before using the selected entity and
    /// adjacency trees. The unrestricted entry shares this authority core.
    pub fn exact_adjacency_structural_revision_admitted<Stop>(
        &self,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
        mut admit: impl FnMut(u64) -> Result<(), Stop>,
    ) -> Result<Result<Option<VersionId>, RelationalBorrowedRecordReadDenial>, Stop> {
        let SnapshotStateBasis::Exact(basis) = &self.basis else {
            return Ok(Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired));
        };
        let Some(entity_work) = self.exact_entity_state_read_work_bound() else {
            return Ok(Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired));
        };
        admit(entity_work)?;
        match self.with_exact_entity_state(anchor, |_, _| ()) {
            Ok(Some(())) => {}
            Ok(None) => return Ok(Ok(None)),
            Err(denial) => return Ok(Err(denial)),
        }
        admit(33)?;
        let direction = match direction {
            RelationalAdjacencyDirection::Outgoing => AdjacencyDirection::Outgoing,
            RelationalAdjacencyDirection::Incoming => AdjacencyDirection::Incoming,
        };
        let Some(partition) = basis.root().get_partition(anchor.partition_id) else {
            return Ok(Ok(None));
        };
        let table = direction.table(partition);
        admit(u64::try_from(table.navigation_work_bound()).unwrap_or(u64::MAX))?;
        let Some(adjacency) = table.get(anchor.slot_index()) else {
            return Ok(Ok(None));
        };
        admit(
            u64::try_from(adjacency.structural_revision_navigation_work_bound())
                .unwrap_or(u64::MAX),
        )?;
        Ok(Ok(adjacency.structural_revision(kind)))
    }

    /// `List` precedes the selected list lookup; each id is borrowed from its
    /// exact pinned substrate. A refusing visitor stops before the next id.
    pub fn try_visit_adjacency_ids<Stop>(
        &self,
        entity: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
        mut visit: impl FnMut(RelationalAdjacencyVisit) -> Result<(), Stop>,
    ) -> Result<(), Stop> {
        visit(RelationalAdjacencyVisit::List)?;
        visit(RelationalAdjacencyVisit::Prepare { work: 33, bytes: 0 })?;
        let Some(root) = self.basis.root() else {
            return Ok(());
        };
        let direction = match direction {
            RelationalAdjacencyDirection::Outgoing => AdjacencyDirection::Outgoing,
            RelationalAdjacencyDirection::Incoming => AdjacencyDirection::Incoming,
        };
        let basis = AdjacencyKindBasis::of_current_version(
            self.basis.root_version() == Some(self.version_id()),
        );
        let table = root
            .get_partition(entity.partition_id)
            .map(|partition| direction.table(partition));
        let adjacency = if let Some(table) = table {
            visit(RelationalAdjacencyVisit::Prepare {
                work: u64::try_from(table.navigation_work_bound()).unwrap_or(u64::MAX),
                bytes: 0,
            })?;
            table.get(entity.slot_index())
        } else {
            None
        };
        if let Some(adjacency) = adjacency {
            visit(RelationalAdjacencyVisit::Prepare {
                work: u64::try_from(adjacency.selected_kind_navigation_work_bound(basis))
                    .unwrap_or(u64::MAX),
                bytes: 0,
            })?;
        }
        let mut ledger = AdjacencyLeaseLedger::default();
        let ids = ledger.lease(adjacency, basis, kind);
        let result = (|| {
            let mut ids = ids
                .try_iter(|work, bytes| visit(RelationalAdjacencyVisit::Prepare { work, bytes }))?;
            while let Some(id) = ids
                .try_next(|work, bytes| visit(RelationalAdjacencyVisit::Prepare { work, bytes }))?
            {
                visit(RelationalAdjacencyVisit::Relation(*id))?;
            }
            Ok(())
        })();
        ledger.settle(self.runtime);
        result
    }
}
