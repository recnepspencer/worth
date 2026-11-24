use crate::storage::data::RelationReadRecord;

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

impl BoundedAdjacencyTruthRead {
    pub(crate) fn owned_bytes(&self) -> u64 {
        super::super::read_footprint::relations(&self.records)
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

/// Operations used by the Relational owners.
pub trait BoundedAdjacencyTruthReadOwnerPort: Sized {
    fn new(records: Vec<RelationReadRecord>, relation_records_examined: usize) -> Self;
}

impl BoundedAdjacencyTruthReadOwnerPort for BoundedAdjacencyTruthRead {
    fn new(records: Vec<RelationReadRecord>, relation_records_examined: usize) -> Self {
        Self {
            records,
            relation_records_examined,
        }
    }
}

/// Operations used by the Relational owners.
pub trait AdjacencyTruthReadLimitExceededOwnerPort: Sized {
    fn new(relation_records_examined: usize, endpoint_records_reserved: usize) -> Self;
}

impl AdjacencyTruthReadLimitExceededOwnerPort for AdjacencyTruthReadLimitExceeded {
    fn new(relation_records_examined: usize, endpoint_records_reserved: usize) -> Self {
        Self {
            relation_records_examined,
            endpoint_records_reserved,
        }
    }
}

pub trait BoundedAdjacencyTruthReadFieldsPort {
    fn owner_records(&self) -> &Vec<RelationReadRecord>;
}

impl BoundedAdjacencyTruthReadFieldsPort for BoundedAdjacencyTruthRead {
    fn owner_records(&self) -> &Vec<RelationReadRecord> {
        &self.records
    }
}
