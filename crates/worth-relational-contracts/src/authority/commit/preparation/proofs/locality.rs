use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::validation::data::groups::InvariantGroupSet;
use crate::validation::data::observation_kind::InvariantObservationKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparationRecordDomain {
    Entity,
    Relation,
    Mixed,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparationPartitionScope {
    AllObserved,
    TouchedPartitions(Arc<[PartitionId]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationReadSetApproximation {
    TouchedOnly,
    SharedCommittedRead,
    FullObservedScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationWriteExclusionClass {
    ReadOnly,
    PublicationExcluded,
    RequiresSingleLaneExecution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationLocalityProof {
    pub observation_scope: InvariantObservationKind,
    pub record_domain: PreparationRecordDomain,
    pub partition_scope: PreparationPartitionScope,
    pub invariant_group_scope: InvariantGroupSet,
    pub read_set_approximation: PreparationReadSetApproximation,
    pub write_exclusion: PreparationWriteExclusionClass,
}
