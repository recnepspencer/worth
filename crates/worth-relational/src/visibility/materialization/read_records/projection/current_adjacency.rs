//! Request-admitted traversal of one authenticated current adjacency bucket.
use super::{RelationalAdjacencyDirection, RelationalAdjacencyVisit};
use crate::identity::data::{EntityId, KindId};
use crate::storage::partition::AdjacencyDirection;
use crate::storage::residency::RelationalRegionHeadView;
use crate::visibility::materialization::read_records::reader::adjacency_work_ledger::AdjacencyLeaseLedger;
pub(super) fn visit<Stop>(
    head: Option<RelationalRegionHeadView<'_>>,
    entity: EntityId,
    kind: KindId,
    direction: RelationalAdjacencyDirection,
    instrumentation: &crate::runtime::RuntimeInstrumentation,
    mut visit: impl FnMut(RelationalAdjacencyVisit) -> Result<(), Stop>,
) -> Result<(), Stop> {
    visit(RelationalAdjacencyVisit::List)?;
    visit(RelationalAdjacencyVisit::Prepare { work: 33, bytes: 0 })?;
    let Some(head) = head else {
        return Ok(());
    };
    let direction = match direction {
        RelationalAdjacencyDirection::Outgoing => AdjacencyDirection::Outgoing,
        RelationalAdjacencyDirection::Incoming => AdjacencyDirection::Incoming,
    };
    let table = head.adjacency(direction);
    visit(RelationalAdjacencyVisit::Prepare {
        work: table.navigation_work_bound() as u64,
        bytes: 0,
    })?;
    let adjacency = table.get(entity.slot_index());
    if let Some(adjacency) = adjacency {
        visit(RelationalAdjacencyVisit::Prepare {
            work: adjacency.navigation_work_bound() as u64,
            bytes: 0,
        })?;
    }
    let mut ledger = AdjacencyLeaseLedger::default();
    let ids = ledger.lease_current(adjacency, kind);
    let result = (|| {
        let mut ids =
            ids.try_iter(|work, bytes| visit(RelationalAdjacencyVisit::Prepare { work, bytes }))?;
        while let Some(id) =
            ids.try_next(|work, bytes| visit(RelationalAdjacencyVisit::Prepare { work, bytes }))?
        {
            visit(RelationalAdjacencyVisit::Relation(*id))?;
        }
        Ok(())
    })();
    ledger.settle_instrumentation(instrumentation);
    result
}
