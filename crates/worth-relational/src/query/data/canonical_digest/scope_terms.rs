#[cfg(test)]
use crate::indexes::data::DerivedIndexGenerationId;
#[cfg(test)]
use crate::query::data::{IndexParityMode, IndexQueryRejectionClass, QueryAccessPath};
use crate::query::data::{
    QueryAccessContract, QueryExecutionShape, QueryLocalityClass, QueryOrderingContract,
    QueryPlanContextId, QueryPlanEvidenceBasis, QueryScope, ReductionDiscipline,
};
use crate::transactions::data::RecordRef;
use crate::visibility::materialization::read_records::{
    ProjectionAspectFilter, ProjectionAspectFilterMode,
};

use super::primitive_terms::{
    encode_descriptor_semantics_version, encode_entity_id, encode_entity_ids, encode_kind_id,
    encode_length_prefixed_aspect_field_locator, encode_length_prefixed_aspect_value,
    encode_optional_kind_id, encode_optional_u32, encode_partition_id, encode_partition_ids,
    encode_schema_version_id, encode_string, encode_u64, encode_usize, encode_version_id,
};

pub(super) fn encode_query_plan_context_id(bytes: &mut Vec<u8>, context_id: &QueryPlanContextId) {
    encode_u64(bytes, context_id.runtime_instance_id);
    encode_u64(bytes, context_id.snapshot_id.0);
    encode_version_id(bytes, context_id.version_id);
    encode_schema_version_id(bytes, context_id.schema_version);
    encode_descriptor_semantics_version(bytes, context_id.descriptor_semantics_version);
    match context_id.evidence_basis {
        QueryPlanEvidenceBasis::CanonicalCommitEnvelope { commit_id } => {
            bytes.push(0);
            encode_u64(bytes, commit_id.0);
        }
        QueryPlanEvidenceBasis::GenesisRuntimeBootstrap => bytes.push(1),
    }
}

pub(super) fn encode_query_scope(bytes: &mut Vec<u8>, scope: &QueryScope) {
    match scope {
        QueryScope::ExplicitTargets { targets } => encode_explicit_targets(bytes, targets),
        QueryScope::EntityKindScan {
            kind_id,
            partition_scope,
        } => encode_kind_scan(bytes, 1, *kind_id, partition_scope.as_deref()),
        QueryScope::RelationKindScan {
            kind_id,
            partition_scope,
        } => encode_kind_scan(bytes, 2, *kind_id, partition_scope.as_deref()),
        QueryScope::EntityFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => encode_field_equals(bytes, 3, field_locator, value, partition_scope.as_deref()),
        QueryScope::EntityFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => encode_field_any_of(bytes, 4, field_locator, values, partition_scope.as_deref()),
        QueryScope::RelationFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => encode_field_equals(bytes, 5, field_locator, value, partition_scope.as_deref()),
        QueryScope::RelationFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => encode_field_any_of(bytes, 6, field_locator, values, partition_scope.as_deref()),
        QueryScope::AspectFilteredEntities {
            kind_id,
            aspect_filter,
            partition_scope,
        } => encode_aspect_filtered_scope(
            bytes,
            7,
            *kind_id,
            aspect_filter,
            partition_scope.as_deref(),
        ),
        QueryScope::AspectFilteredRelations {
            kind_id,
            aspect_filter,
            partition_scope,
        } => encode_aspect_filtered_scope(
            bytes,
            8,
            *kind_id,
            aspect_filter,
            partition_scope.as_deref(),
        ),
        QueryScope::OutgoingNeighborhood {
            seeds,
            relation_kind_scope,
        } => encode_neighborhood_scope(bytes, 9, seeds, relation_kind_scope.as_deref(), None),
        QueryScope::IncomingNeighborhood {
            seeds,
            relation_kind_scope,
        } => encode_neighborhood_scope(bytes, 10, seeds, relation_kind_scope.as_deref(), None),
        QueryScope::ConnectivityTraversal {
            seeds,
            relation_kind_scope,
            max_depth,
        } => encode_neighborhood_scope(
            bytes,
            11,
            seeds,
            relation_kind_scope.as_deref(),
            Some(*max_depth),
        ),
    }
}

pub(super) fn encode_query_locality_class(bytes: &mut Vec<u8>, locality: &QueryLocalityClass) {
    match locality {
        QueryLocalityClass::SinglePartition { partition_id } => {
            bytes.push(0);
            encode_partition_id(bytes, *partition_id);
        }
        QueryLocalityClass::PartitionBounded { partitions } => {
            bytes.push(1);
            encode_partition_ids(bytes, partitions);
        }
        QueryLocalityClass::CrossPartitionTraversal => bytes.push(2),
    }
}

#[cfg(test)]
pub(super) fn encode_query_access_path(bytes: &mut Vec<u8>, access_path: &QueryAccessPath) {
    match access_path {
        QueryAccessPath::AuthoritativeStorage => bytes.push(0),
        QueryAccessPath::DerivedIndexGeneration { generation_id } => {
            bytes.push(1);
            encode_derived_index_generation_id(bytes, *generation_id);
        }
        QueryAccessPath::DerivedIndexRejectedStorageRead { rejection } => {
            bytes.push(2);
            encode_index_query_rejection_class(bytes, rejection);
        }
    }
}

fn encode_explicit_targets(bytes: &mut Vec<u8>, targets: &[RecordRef]) {
    bytes.push(0);
    encode_usize(bytes, targets.len());
    for target in targets {
        match target {
            RecordRef::Entity(entity_id) => {
                bytes.push(0);
                encode_entity_id(bytes, *entity_id);
            }
            RecordRef::Relation(relation_id) => {
                bytes.push(1);
                super::primitive_terms::encode_relation_id(bytes, *relation_id);
            }
        }
    }
}

fn encode_kind_scan(
    bytes: &mut Vec<u8>,
    tag: u8,
    kind_id: crate::identity::data::KindId,
    partition_scope: Option<&[crate::identity::data::PartitionId]>,
) {
    bytes.push(tag);
    encode_kind_id(bytes, kind_id);
    encode_partition_scope(bytes, partition_scope);
}

fn encode_field_equals(
    bytes: &mut Vec<u8>,
    tag: u8,
    field_locator: &worth_foundational::facade::AspectFieldLocator,
    value: &worth_foundational::facade::AspectValue,
    partition_scope: Option<&[crate::identity::data::PartitionId]>,
) {
    bytes.push(tag);
    encode_length_prefixed_aspect_field_locator(bytes, field_locator);
    encode_length_prefixed_aspect_value(bytes, value);
    encode_partition_scope(bytes, partition_scope);
}

fn encode_field_any_of(
    bytes: &mut Vec<u8>,
    tag: u8,
    field_locator: &worth_foundational::facade::AspectFieldLocator,
    values: &[worth_foundational::facade::AspectValue],
    partition_scope: Option<&[crate::identity::data::PartitionId]>,
) {
    bytes.push(tag);
    encode_length_prefixed_aspect_field_locator(bytes, field_locator);
    encode_usize(bytes, values.len());
    for value in values {
        encode_length_prefixed_aspect_value(bytes, value);
    }
    encode_partition_scope(bytes, partition_scope);
}

fn encode_aspect_filtered_scope(
    bytes: &mut Vec<u8>,
    tag: u8,
    kind_id: Option<crate::identity::data::KindId>,
    aspect_filter: &ProjectionAspectFilter,
    partition_scope: Option<&[crate::identity::data::PartitionId]>,
) {
    bytes.push(tag);
    encode_optional_kind_id(bytes, kind_id);
    encode_aspect_filter(bytes, aspect_filter);
    encode_partition_scope(bytes, partition_scope);
}

fn encode_neighborhood_scope(
    bytes: &mut Vec<u8>,
    tag: u8,
    seeds: &[crate::identity::data::EntityId],
    relation_kind_scope: Option<&[crate::identity::data::KindId]>,
    max_depth: Option<Option<u32>>,
) {
    bytes.push(tag);
    encode_entity_ids(bytes, seeds);
    encode_kind_scope(bytes, relation_kind_scope);
    if let Some(max_depth) = max_depth {
        encode_optional_u32(bytes, max_depth);
    }
}

fn encode_partition_scope(
    bytes: &mut Vec<u8>,
    partitions: Option<&[crate::identity::data::PartitionId]>,
) {
    match partitions {
        Some(partitions) => {
            bytes.push(1);
            encode_partition_ids(bytes, partitions);
        }
        None => bytes.push(0),
    }
}

fn encode_kind_scope(bytes: &mut Vec<u8>, kinds: Option<&[crate::identity::data::KindId]>) {
    match kinds {
        Some(kinds) => {
            bytes.push(1);
            encode_usize(bytes, kinds.len());
            for kind in kinds {
                encode_kind_id(bytes, *kind);
            }
        }
        None => bytes.push(0),
    }
}

fn encode_aspect_filter(bytes: &mut Vec<u8>, filter: &ProjectionAspectFilter) {
    match filter.mode() {
        ProjectionAspectFilterMode::Any => bytes.push(0),
        ProjectionAspectFilterMode::All => bytes.push(1),
    }
    encode_usize(bytes, filter.projection_scope().requirements().len());
    for requirement in filter.projection_scope().requirements() {
        encode_string(bytes, requirement.aspect_key().as_str());
        if requirement.mask().is_whole_aspect() {
            bytes.push(0);
            continue;
        }
        bytes.push(1);
        encode_usize(bytes, requirement.mask().paths().len());
        for path in requirement.mask().paths() {
            encode_usize(bytes, path.fields().len());
            for field in path.fields() {
                encode_string(bytes, field.as_str());
            }
        }
    }
}

pub(super) fn encode_query_execution_shape(
    bytes: &mut Vec<u8>,
    execution_shape: QueryExecutionShape,
) {
    bytes.push(match execution_shape {
        QueryExecutionShape::SingleEntity => 0,
        QueryExecutionShape::BulkPacketized => 1,
    });
}

pub(super) fn encode_reduction_discipline(bytes: &mut Vec<u8>, reduction: ReductionDiscipline) {
    bytes.push(match reduction {
        ReductionDiscipline::DeterministicMerge => 0,
    });
}

pub(super) fn encode_query_ordering_contract(bytes: &mut Vec<u8>, ordering: QueryOrderingContract) {
    bytes.push(match ordering {
        QueryOrderingContract::CanonicalEntityIdOrder => 0,
        QueryOrderingContract::CanonicalRelationIdOrder => 1,
        QueryOrderingContract::CanonicalRecordRefOrder => 2,
        QueryOrderingContract::CanonicalTraversalOrder => 3,
    });
}

pub(super) fn encode_query_access_contract(
    bytes: &mut Vec<u8>,
    access_contract: QueryAccessContract,
) {
    bytes.push(match access_contract {
        QueryAccessContract::AuthoritativeStorageOnly => 0,
        QueryAccessContract::DerivedIndexWithStorageParity => 1,
    });
}

#[cfg(test)]
pub(super) fn encode_index_parity_mode(bytes: &mut Vec<u8>, parity_mode: IndexParityMode) {
    bytes.push(match parity_mode {
        IndexParityMode::ProductionAdmissibility => 0,
        IndexParityMode::SampledParity => 1,
        IndexParityMode::CertificationParity => 2,
    });
}

#[cfg(test)]
fn encode_index_query_rejection_class(bytes: &mut Vec<u8>, rejection: &IndexQueryRejectionClass) {
    bytes.push(match rejection {
        IndexQueryRejectionClass::MissingGeneration => 0,
        IndexQueryRejectionClass::UnsupportedVersion => 1,
        IndexQueryRejectionClass::UnsupportedBranch => 2,
        IndexQueryRejectionClass::CorruptIndexEntries => 3,
        IndexQueryRejectionClass::UnsupportedScope => 4,
        IndexQueryRejectionClass::UnsupportedOrderingContract => 5,
    });
}

#[cfg(test)]
fn encode_derived_index_generation_id(bytes: &mut Vec<u8>, id: DerivedIndexGenerationId) {
    encode_u64(bytes, id.0);
}
