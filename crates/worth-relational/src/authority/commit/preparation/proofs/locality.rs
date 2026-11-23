use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::validation::data::groups::InvariantGroupSet;
use crate::validation::data::observation_kind::InvariantObservationKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreparationRecordDomain {
    Entity,
    Relation,
    Mixed,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreparationPartitionScope {
    AllObserved,
    TouchedPartitions(Arc<[PartitionId]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreparationReadSetApproximation {
    TouchedOnly,
    SharedCommittedRead,
    FullObservedScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreparationWriteExclusionClass {
    ReadOnly,
    PublicationExcluded,
    RequiresSingleLaneExecution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparationLocalityProof {
    pub(crate) observation_scope: InvariantObservationKind,
    pub(crate) record_domain: PreparationRecordDomain,
    pub(crate) partition_scope: PreparationPartitionScope,
    pub(crate) invariant_group_scope: InvariantGroupSet,
    pub(crate) read_set_approximation: PreparationReadSetApproximation,
    pub(crate) write_exclusion: PreparationWriteExclusionClass,
}
