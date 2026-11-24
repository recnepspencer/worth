use super::adjacency_work_ledger::AdjacencyLeaseLedger;
use super::kind_scan_visibility::KindScanVisibility;
use super::truth_record_access::sort_authoritative_relation_records;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};
use crate::{storage::data::RelationReadRecord, storage::overlay::PartitionAccess};

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

#[path = "truth_adjacency/runtime_reads.rs"]
mod runtime_reads;

impl BoundedAdjacencyTruthRead {
    pub(crate) fn owned_bytes(&self) -> u64 {
        super::super::read_footprint::relations(&self.records)
    }
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
