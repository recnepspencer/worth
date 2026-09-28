//! One entity's adjacency, read on the view's own branch root.
//!
//! The version-addressed reader answers from the runtime's shared partition
//! edition, so a branch reading at its own version can observe a sibling's
//! commit whose version is older. This read resolves the adjacency list and
//! every relation record against the branch root instead, while charging work
//! exactly as the version-addressed read does: one unit per examined relation
//! and one per admitted record.

use crate::identity::data::{EntityId, KindId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;
use crate::visibility::materialization::read_records::reader::adjacency_work_ledger::AdjacencyLeaseLedger;
use crate::visibility::materialization::read_records::reader::{
    AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead,
};

impl VisibilityProjectionView<'_> {
    pub fn bounded_outgoing_relations_of_kind(
        &self,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded> {
        self.bounded_entity_relations_of_kind(
            entity_id,
            kind_id,
            AdjacencyDirection::Outgoing,
            maximum_work_units,
        )
    }

    pub fn bounded_incoming_relations_of_kind(
        &self,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded> {
        self.bounded_entity_relations_of_kind(
            entity_id,
            kind_id,
            AdjacencyDirection::Incoming,
            maximum_work_units,
        )
    }

    fn bounded_entity_relations_of_kind(
        &self,
        entity_id: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: usize,
    ) -> Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded> {
        let Some(root) = self.basis.root() else {
            return Ok(BoundedAdjacencyTruthRead::new(Vec::new(), 0));
        };
        let adjacency_basis = AdjacencyKindBasis::of_current_version(
            self.basis.root_version() == Some(self.version_id()),
        );
        let mut ledger = AdjacencyLeaseLedger::default();
        let relation_ids = ledger.lease(
            root.get_partition(entity_id.partition_id)
                .and_then(|partition| direction.table(partition).get(entity_id.slot_index())),
            adjacency_basis,
            kind_id,
        );
        let mut records = Vec::new();
        let mut examined = 0_usize;
        for relation_id in relation_ids.iter().copied() {
            if examined + records.len() == maximum_work_units {
                ledger.settle(self.runtime);
                return Err(AdjacencyTruthReadLimitExceeded::new(
                    examined,
                    records.len(),
                ));
            }
            examined += 1;
            let Some(record) = self.authoritative_relation_record(relation_id) else {
                continue;
            };
            if record.kind.kind_id != kind_id
                || record.lifecycle != RecordLifecycleState::Live
                || !direction.matches_endpoint(&record, entity_id)
            {
                continue;
            }
            if examined + records.len() == maximum_work_units {
                ledger.settle(self.runtime);
                return Err(AdjacencyTruthReadLimitExceeded::new(
                    examined,
                    records.len(),
                ));
            }
            records.push(record);
        }
        ledger.settle(self.runtime);
        records.sort_by_key(relation_identity_order_key);
        Ok(BoundedAdjacencyTruthRead::new(records, examined))
    }
}
