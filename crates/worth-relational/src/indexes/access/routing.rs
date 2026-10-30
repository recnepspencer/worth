use crate::history::data::BranchId;
use crate::indexes::data::{DerivedIndexEntries, DerivedIndexGeneration, DerivedIndexKind};
use crate::query::data::{
    IndexQueryRejectionClass, PlannedQueryPacket, QueryAccessContract, QueryAccessPath, QueryScope,
    SnapshotPinnedQueryPlan,
};
use crate::runtime::RelationalRuntime;

const SAMPLED_PARITY_MODULUS: u128 = 8;
const SAMPLED_PARITY_REMAINDER: u128 = 0;

pub(crate) fn admissible_access_path(
    runtime: &RelationalRuntime,
    plan: &SnapshotPinnedQueryPlan,
) -> QueryAccessPath {
    if plan.packet.access_contract == QueryAccessContract::AuthoritativeStorageOnly {
        return QueryAccessPath::AuthoritativeStorage;
    }

    let branch_id = plan.snapshot.branch_id().clone();
    let Some(generation) = candidate_generation_for_packet(runtime, &plan.packet, &branch_id)
    else {
        return QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: if matching_index_definition_exists(runtime, &plan.packet) {
                IndexQueryRejectionClass::MissingGeneration
            } else if runtime
                .indexes
                .any_generation_at_or_before(plan.snapshot.version_id)
            {
                IndexQueryRejectionClass::UnsupportedScope
            } else {
                IndexQueryRejectionClass::MissingGeneration
            },
        };
    };

    match index_rejection_for_packet(runtime, &plan.packet, &generation, &branch_id) {
        Some(rejection) => QueryAccessPath::DerivedIndexRejectedStorageRead { rejection },
        None => QueryAccessPath::DerivedIndexGeneration {
            generation_id: generation.generation_id,
        },
    }
}

pub(crate) fn should_verify_sampled_parity(
    plan: &SnapshotPinnedQueryPlan,
    generation_id: crate::indexes::data::DerivedIndexGenerationId,
) -> bool {
    let sample_key = plan.packet.plan_key.0
        ^ ((generation_id.0 as u128) << 64)
        ^ (plan.snapshot.version_id.0 as u128);
    sample_key % SAMPLED_PARITY_MODULUS == SAMPLED_PARITY_REMAINDER
}

fn index_rejection_for_packet(
    runtime: &RelationalRuntime,
    packet: &PlannedQueryPacket,
    generation: &DerivedIndexGeneration,
    branch_id: &BranchId,
) -> Option<IndexQueryRejectionClass> {
    if generation.status != crate::indexes::data::DerivedIndexPublicationStatus::Published {
        return Some(IndexQueryRejectionClass::CorruptIndexEntries);
    }
    if generation.applicability.branch_id != *branch_id
        && runtime
            .indexes
            .definition(generation.index_id)
            .is_some_and(|definition| definition.branch_scoped)
    {
        return Some(IndexQueryRejectionClass::UnsupportedBranch);
    }
    if generation.applicability.version_id != packet.context_id.version_id {
        return Some(IndexQueryRejectionClass::UnsupportedVersion);
    }
    if generation.applicability.schema_version != packet.context_id.schema_version {
        return Some(IndexQueryRejectionClass::UnsupportedVersion);
    }
    match &packet.scope {
        QueryScope::EntityFieldEquals { .. } | QueryScope::EntityFieldAnyOf { .. } => {
            if !matches!(
                packet.ordering,
                crate::query::data::QueryOrderingContract::CanonicalRecordRefOrder
                    | crate::query::data::QueryOrderingContract::CanonicalEntityIdOrder
            ) {
                return Some(IndexQueryRejectionClass::UnsupportedOrderingContract);
            }
        }
        QueryScope::RelationFieldEquals { .. } | QueryScope::RelationFieldAnyOf { .. } => {
            if !matches!(
                packet.ordering,
                crate::query::data::QueryOrderingContract::CanonicalRecordRefOrder
                    | crate::query::data::QueryOrderingContract::CanonicalRelationIdOrder
            ) {
                return Some(IndexQueryRejectionClass::UnsupportedOrderingContract);
            }
        }
        _ => {
            if !matches!(
                packet.ordering,
                crate::query::data::QueryOrderingContract::CanonicalRecordRefOrder
                    | crate::query::data::QueryOrderingContract::CanonicalEntityIdOrder
                    | crate::query::data::QueryOrderingContract::CanonicalRelationIdOrder
            ) {
                return Some(IndexQueryRejectionClass::UnsupportedOrderingContract);
            }
        }
    }
    let definition = runtime.indexes.definition(generation.index_id);
    match (&packet.scope, &generation.entries, definition.as_deref()) {
        (
            QueryScope::EntityFieldEquals { field_locator, .. },
            DerivedIndexEntries::EntityField(_),
            Some(definition),
        )
        | (
            QueryScope::EntityFieldAnyOf { field_locator, .. },
            DerivedIndexEntries::EntityField(_),
            Some(definition),
        ) => match &definition.kind {
            DerivedIndexKind::EntityField {
                field_locator: indexed_field_locator,
            } if indexed_field_locator == field_locator => None,
            _ => Some(IndexQueryRejectionClass::UnsupportedScope),
        },
        (
            QueryScope::RelationFieldEquals { field_locator, .. },
            DerivedIndexEntries::RelationField(_),
            Some(definition),
        )
        | (
            QueryScope::RelationFieldAnyOf { field_locator, .. },
            DerivedIndexEntries::RelationField(_),
            Some(definition),
        ) => match &definition.kind {
            DerivedIndexKind::RelationField {
                field_locator: indexed_field_locator,
            } if indexed_field_locator == field_locator => None,
            _ => Some(IndexQueryRejectionClass::UnsupportedScope),
        },
        _ => Some(IndexQueryRejectionClass::UnsupportedScope),
    }
}

fn candidate_generation_for_packet(
    runtime: &RelationalRuntime,
    packet: &PlannedQueryPacket,
    branch_id: &BranchId,
) -> Option<std::sync::Arc<DerivedIndexGeneration>> {
    match &packet.scope {
        QueryScope::EntityFieldEquals { field_locator, .. }
        | QueryScope::EntityFieldAnyOf { field_locator, .. } => runtime
            .indexes
            .definitions()
            .into_iter()
            .filter(|definition| {
                matches!(
                    &definition.kind,
                    DerivedIndexKind::EntityField { field_locator: indexed_field_locator }
                        if indexed_field_locator == field_locator
                )
            })
            .filter_map(|definition| {
                runtime.indexes.candidate_generation(
                    definition.index_id,
                    definition.branch_scoped.then_some(branch_id),
                    packet.context_id.version_id,
                    packet.context_id.schema_version,
                )
            })
            .max_by(|left, right| {
                generation_preference(runtime, left.as_ref(), packet, branch_id)
                    .cmp(&generation_preference(
                        runtime,
                        right.as_ref(),
                        packet,
                        branch_id,
                    ))
                    .then_with(|| left.generation_id.cmp(&right.generation_id))
            }),
        QueryScope::RelationFieldEquals { field_locator, .. }
        | QueryScope::RelationFieldAnyOf { field_locator, .. } => runtime
            .indexes
            .definitions()
            .into_iter()
            .filter(|definition| {
                matches!(
                    &definition.kind,
                    DerivedIndexKind::RelationField { field_locator: indexed_field_locator }
                        if indexed_field_locator == field_locator
                )
            })
            .filter_map(|definition| {
                runtime.indexes.candidate_generation(
                    definition.index_id,
                    definition.branch_scoped.then_some(branch_id),
                    packet.context_id.version_id,
                    packet.context_id.schema_version,
                )
            })
            .max_by(|left, right| {
                generation_preference(runtime, left.as_ref(), packet, branch_id)
                    .cmp(&generation_preference(
                        runtime,
                        right.as_ref(),
                        packet,
                        branch_id,
                    ))
                    .then_with(|| left.generation_id.cmp(&right.generation_id))
            }),
        _ => None,
    }
}

fn matching_index_definition_exists(
    runtime: &RelationalRuntime,
    packet: &PlannedQueryPacket,
) -> bool {
    match &packet.scope {
        QueryScope::EntityFieldEquals { field_locator, .. }
        | QueryScope::EntityFieldAnyOf { field_locator, .. } => {
            runtime.indexes.definitions().iter().any(|definition| {
                matches!(
                    &definition.kind,
                    DerivedIndexKind::EntityField { field_locator: indexed_field_locator }
                        if indexed_field_locator == field_locator
                )
            })
        }
        QueryScope::RelationFieldEquals { field_locator, .. }
        | QueryScope::RelationFieldAnyOf { field_locator, .. } => {
            runtime.indexes.definitions().iter().any(|definition| {
                matches!(
                    &definition.kind,
                    DerivedIndexKind::RelationField { field_locator: indexed_field_locator }
                        if indexed_field_locator == field_locator
                )
            })
        }
        _ => false,
    }
}

fn generation_preference(
    runtime: &RelationalRuntime,
    generation: &DerivedIndexGeneration,
    packet: &PlannedQueryPacket,
    branch_id: &BranchId,
) -> (bool, bool, bool, bool, crate::identity::data::VersionId) {
    let branch_applicable =
        runtime
            .indexes
            .definition(generation.index_id)
            .is_none_or(|definition| {
                !definition.branch_scoped || generation.applicability.branch_id == *branch_id
            });
    let version_applicable = generation.applicability.version_id == packet.context_id.version_id;
    let schema_applicable =
        generation.applicability.schema_version == packet.context_id.schema_version;
    let published =
        generation.status == crate::indexes::data::DerivedIndexPublicationStatus::Published;
    (
        published,
        branch_applicable,
        version_applicable,
        schema_applicable,
        generation.applicability.version_id,
    )
}
