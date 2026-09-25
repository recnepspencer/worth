use std::collections::BTreeSet;

use crate::identity::data::{EntityId, KindId};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;
use crate::visibility::materialization::read_records::reader::adjacency_work_ledger::AdjacencyLeaseLedger;
use crate::visibility::materialization::read_records::reader::{
    BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded,
};

impl VisibilityProjectionView<'_> {
    pub fn bounded_outgoing_relations_for_frontier(
        &self,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded> {
        self.bounded_relations_for_frontier(
            entity_ids,
            kind_id,
            AdjacencyDirection::Outgoing,
            maximum_work_units,
            false,
        )
    }

    pub fn bounded_incoming_relations_for_frontier(
        &self,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded> {
        self.bounded_relations_for_frontier(
            entity_ids,
            kind_id,
            AdjacencyDirection::Incoming,
            maximum_work_units,
            false,
        )
    }

    /// Index maintenance mirrors full-build relation visibility, including
    /// audit-retained edges. Ordinary frontier queries remain Live-only.
    pub(crate) fn bounded_index_relations_for_frontier(
        &self,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        outgoing: bool,
        maximum_work_units: usize,
    ) -> Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded> {
        self.bounded_relations_for_frontier(
            entity_ids,
            kind_id,
            if outgoing {
                AdjacencyDirection::Outgoing
            } else {
                AdjacencyDirection::Incoming
            },
            maximum_work_units,
            true,
        )
    }

    fn bounded_relations_for_frontier(
        &self,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: usize,
        include_audit: bool,
    ) -> Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded> {
        let Some(root) = self.basis.root() else {
            let mut lists = 0;
            for _ in entity_ids {
                charge(maximum_work_units, lists, 0, 0)?;
                lists += 1;
            }
            return Ok(BoundedFrontierAdjacencyTruthRead {
                records: Vec::new(),
                adjacency_lists_read: lists,
                relation_records_examined: 0,
            });
        };
        let adjacency_basis = AdjacencyKindBasis::of_current_version(
            self.basis.root_version() == Some(self.version_id()),
        );
        let mut ledger = AdjacencyLeaseLedger::default();
        let mut records = Vec::new();
        let mut lists = 0_usize;
        let mut examined = 0_usize;
        for entity_id in entity_ids {
            if let Err(exceeded) = charge(maximum_work_units, lists, examined, records.len()) {
                ledger.settle(self.runtime);
                return Err(exceeded);
            }
            lists += 1;
            let relation_ids = ledger.lease(
                root.get_partition(entity_id.partition_id)
                    .and_then(|partition| direction.table(partition).get(entity_id.slot_index())),
                adjacency_basis,
                kind_id,
            );
            for relation_id in relation_ids.iter().copied() {
                if let Err(exceeded) = charge(maximum_work_units, lists, examined, records.len()) {
                    ledger.settle(self.runtime);
                    return Err(exceeded);
                }
                examined += 1;
                let Some(record) = self.authoritative_relation_record(relation_id) else {
                    continue;
                };
                if record.kind.kind_id != kind_id
                    || !(record.lifecycle == crate::storage::data::RecordLifecycleState::Live
                        || (include_audit && record.lifecycle
                            == crate::storage::data::RecordLifecycleState::RetainedDanglingForAudit))
                    || !direction.matches_endpoint(&record, *entity_id)
                {
                    continue;
                }
                if let Err(exceeded) = charge(maximum_work_units, lists, examined, records.len()) {
                    ledger.settle(self.runtime);
                    return Err(exceeded);
                }
                records.push(record);
            }
        }
        ledger.settle(self.runtime);
        records.sort_by_key(relation_identity_order_key);
        Ok(BoundedFrontierAdjacencyTruthRead {
            records,
            adjacency_lists_read: lists,
            relation_records_examined: examined,
        })
    }
}

fn charge(
    maximum: usize,
    lists: usize,
    examined: usize,
    endpoints: usize,
) -> Result<(), FrontierAdjacencyTruthReadLimitExceeded> {
    if lists.saturating_add(examined).saturating_add(endpoints) >= maximum {
        Err(FrontierAdjacencyTruthReadLimitExceeded::new(
            lists, examined, endpoints,
        ))
    } else {
        Ok(())
    }
}
