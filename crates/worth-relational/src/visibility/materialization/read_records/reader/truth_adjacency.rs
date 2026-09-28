use super::adjacency_work_ledger::AdjacencyLeaseLedger;
use super::truth_record_access::sort_authoritative_relation_records;
use super::*;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

#[derive(Debug)]
pub struct BoundedAdjacencyTruthRead {
    records: Vec<RelationReadRecord>,
    relation_records_examined: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdjacencyTruthReadLimitExceeded {
    relation_records_examined: usize,
    endpoint_records_reserved: usize,
}

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn outgoing_relations_of_kind_at_version(
        &self,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<RelationReadRecord> {
        self.bounded_relations_of_kind_at_version(
            entity_id,
            kind_id,
            version_id,
            AdjacencyDirection::Outgoing,
            usize::MAX,
        )
        .expect("an unbounded adjacency read cannot exhaust usize::MAX work")
        .into_records()
    }

    pub fn incoming_relations_of_kind_at_version(
        &self,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<RelationReadRecord> {
        self.bounded_relations_of_kind_at_version(
            entity_id,
            kind_id,
            version_id,
            AdjacencyDirection::Incoming,
            usize::MAX,
        )
        .expect("an unbounded adjacency read cannot exhaust usize::MAX work")
        .into_records()
    }

    pub fn bounded_outgoing_relations_of_kind_at_version(
        &self,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded> {
        self.bounded_relations_of_kind_at_version(
            entity_id,
            kind_id,
            version_id,
            AdjacencyDirection::Outgoing,
            maximum_work_units,
        )
    }

    pub fn bounded_incoming_relations_of_kind_at_version(
        &self,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded> {
        self.bounded_relations_of_kind_at_version(
            entity_id,
            kind_id,
            version_id,
            AdjacencyDirection::Incoming,
            maximum_work_units,
        )
    }

    fn bounded_relations_of_kind_at_version(
        &self,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        direction: AdjacencyDirection,
        maximum_work_units: usize,
    ) -> Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded> {
        let slot = entity_id.slot_index();
        let basis =
            AdjacencyKindBasis::of_current_version(version_id == self.runtime.current_version_id());
        // One edition, pinned for the whole traversal. Every record below is
        // resolved against it, so a fanout of degree D costs one substrate
        // acquisition rather than D of them, and the answer describes a single
        // consistent substrate rather than however many editions raced past.
        let edition = self.runtime.acquire_partition_edition();
        let mut ledger = AdjacencyLeaseLedger::default();
        // Leased, never copied: the bound must be applied to the fanout, not
        // after materializing it.
        let relation_ids = ledger.lease(
            edition
                .partition(entity_id.partition_id)
                .and_then(|partition| direction.table(partition).get(slot)),
            basis,
            kind_id,
        );
        let mut records = Vec::new();
        let mut work_units = 0_usize;
        let mut relation_records_examined = 0_usize;
        for relation_id in relation_ids.iter().copied() {
            if work_units == maximum_work_units {
                ledger.settle(self.runtime);
                return Err(AdjacencyTruthReadLimitExceeded::new(
                    relation_records_examined,
                    records.len(),
                ));
            }
            work_units += 1;
            relation_records_examined += 1;
            let Some(record) = self.authoritative_relation_record_for_id_at_version(
                &edition,
                relation_id,
                version_id,
            ) else {
                continue;
            };
            if record.kind.kind_id != kind_id
                || record.lifecycle != crate::storage::data::RecordLifecycleState::Live
                || !direction.matches_endpoint(&record, entity_id)
            {
                continue;
            }
            if work_units == maximum_work_units {
                ledger.settle(self.runtime);
                return Err(AdjacencyTruthReadLimitExceeded::new(
                    relation_records_examined,
                    records.len(),
                ));
            }
            work_units += 1;
            records.push(record);
        }
        ledger.settle(self.runtime);
        sort_authoritative_relation_records(&mut records);
        Ok(BoundedAdjacencyTruthRead {
            records,
            relation_records_examined,
        })
    }
}

impl BoundedAdjacencyTruthRead {
    pub(crate) const fn new(
        records: Vec<RelationReadRecord>,
        relation_records_examined: usize,
    ) -> Self {
        Self {
            records,
            relation_records_examined,
        }
    }

    pub const fn relation_records_examined(&self) -> usize {
        self.relation_records_examined
    }

    pub const fn endpoint_records_reserved(&self) -> usize {
        self.records.len()
    }

    pub const fn work_units(&self) -> usize {
        self.relation_records_examined + self.records.len()
    }

    pub fn into_records(self) -> Vec<RelationReadRecord> {
        self.records
    }
}

impl AdjacencyTruthReadLimitExceeded {
    pub(crate) const fn new(
        relation_records_examined: usize,
        endpoint_records_reserved: usize,
    ) -> Self {
        Self {
            relation_records_examined,
            endpoint_records_reserved,
        }
    }

    pub const fn relation_records_examined(self) -> usize {
        self.relation_records_examined
    }

    pub const fn endpoint_records_reserved(self) -> usize {
        self.endpoint_records_reserved
    }

    pub const fn consumed_work_units(self) -> usize {
        self.relation_records_examined + self.endpoint_records_reserved
    }
}
