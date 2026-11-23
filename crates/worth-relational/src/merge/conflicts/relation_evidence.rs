use std::sync::Arc;

use crate::merge::conflicts::ancestor_record_basis::{
    AncestorRecordBasisContext, AncestorRelationRecordBasis,
};
use crate::merge::data::{
    EndpointContinuityClass, RelationConflictEvidence, RelationConflictPropagation,
    RelationContinuityClass, VisibleMergeRecord, VisibleMergeRecordKind,
};
use crate::storage::data::RelationReadRecord;
use crate::transactions::data::primitives::RecordRef;

pub(super) fn relation_conflict_evidence(
    record: &VisibleMergeRecord,
    target_record: Option<&VisibleMergeRecord>,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
) -> Option<RelationConflictEvidence> {
    if record.record_kind != VisibleMergeRecordKind::Relation {
        return None;
    }
    let base =
        ancestor_basis.relation_basis(record, target_record.map(|record| &record.record_ref));
    let source = record.source_relation.as_ref();
    let target = target_record
        .and_then(|record| record.target_relation.as_ref())
        .or(record.target_relation.as_ref());

    let endpoint_continuity = match (source, target, base) {
        (Some(source), Some(target), _) => endpoint_continuity_between(source, target),
        (Some(source), None, Some(base)) => endpoint_continuity_from_ancestor(&base, source),
        (None, Some(target), Some(base)) => endpoint_continuity_from_ancestor(&base, target),
        _ => EndpointContinuityClass::EndpointsStable,
    };
    let relation_continuity = match endpoint_continuity {
        EndpointContinuityClass::EndpointsStable => {
            RelationContinuityClass::PreserveRelationIdentity
        }
        EndpointContinuityClass::SourceEndpointRewired
        | EndpointContinuityClass::TargetEndpointRewired
        | EndpointContinuityClass::BothEndpointsRewired => {
            RelationContinuityClass::RetireAndIntroduceSuccessor
        }
    };
    let propagation = match endpoint_continuity {
        EndpointContinuityClass::EndpointsStable => RelationConflictPropagation::RelationLocalOnly,
        _ => RelationConflictPropagation::RelationLocalRewireCandidate,
    };
    Some(RelationConflictEvidence {
        endpoint_continuity,
        relation_continuity,
        propagation,
        topology_neighborhood_records: Arc::from(Vec::<RecordRef>::new()),
        topology_neighborhood_rewired_records: Arc::from(Vec::<RecordRef>::new()),
        topology_region_conflict_reason: None,
    })
}

fn endpoint_continuity_between(
    left: &RelationReadRecord,
    right: &RelationReadRecord,
) -> EndpointContinuityClass {
    match (left.source == right.source, left.target == right.target) {
        (true, true) => EndpointContinuityClass::EndpointsStable,
        (false, true) => EndpointContinuityClass::SourceEndpointRewired,
        (true, false) => EndpointContinuityClass::TargetEndpointRewired,
        (false, false) => EndpointContinuityClass::BothEndpointsRewired,
    }
}

fn endpoint_continuity_from_ancestor(
    ancestor: &AncestorRelationRecordBasis<'_>,
    right: &RelationReadRecord,
) -> EndpointContinuityClass {
    match (
        ancestor.source_endpoint() == right.source,
        ancestor.target_endpoint() == right.target,
    ) {
        (true, true) => EndpointContinuityClass::EndpointsStable,
        (false, true) => EndpointContinuityClass::SourceEndpointRewired,
        (true, false) => EndpointContinuityClass::TargetEndpointRewired,
        (false, false) => EndpointContinuityClass::BothEndpointsRewired,
    }
}
