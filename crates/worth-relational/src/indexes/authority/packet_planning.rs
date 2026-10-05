use crate::authority::commit::preparation::packets::index::{
    IndexFragmentIdentity, IndexPreparationHeader, IndexPreparationPacket,
};
use crate::authority::commit::preparation::planning::strategy::{
    strategy_for_parallel_packets, PreparationStrategy,
};
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationLocalityProof, PreparationPartitionScope, PreparationReadSetApproximation,
    PreparationRecordDomain, PreparationWriteExclusionClass,
};
use crate::authority::commit::preparation::reduction::keys::IndexReductionKey;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::runtime::RelationalRuntime;
use crate::validation::data::InvariantGroupSet;
use worth_execution::ExecutionResourceLease;

mod checked;
pub(super) use checked::plan_index_packets_checked;

pub(super) fn planned_index_definitions(
    runtime: &RelationalRuntime,
    index_ids: &[DerivedIndexId],
) -> (Vec<DerivedIndexDefinition>, Vec<DerivedIndexId>) {
    let mut definitions = Vec::new();
    let mut missing_indexes = Vec::new();

    for index_id in index_ids {
        if let Some(definition) = runtime.indexes.definition(*index_id) {
            definitions.push(definition.as_ref().clone());
        } else {
            missing_indexes.push(*index_id);
        }
    }

    (definitions, missing_indexes)
}

pub(super) fn choose_index_preparation_strategy(
    _runtime: &RelationalRuntime,
    lease: Option<&ExecutionResourceLease>,
    packet_count: usize,
) -> PreparationStrategy {
    strategy_for_parallel_packets(lease, packet_count)
}

pub(super) fn plan_index_packets(
    definitions: Vec<DerivedIndexDefinition>,
) -> Vec<IndexPreparationPacket> {
    definitions
        .into_iter()
        .enumerate()
        .map(|(packet_index, definition)| packet_for_definition(packet_index, definition))
        .collect()
}

fn packet_for_definition(
    packet_index: usize,
    definition: DerivedIndexDefinition,
) -> IndexPreparationPacket {
    let record_domain = match definition.kind {
        DerivedIndexKind::EntityField { .. } => PreparationRecordDomain::Entity,
        DerivedIndexKind::RelationField { .. } => PreparationRecordDomain::Relation,
        DerivedIndexKind::RelatedEntityOrdering { .. } => PreparationRecordDomain::Mixed,
        DerivedIndexKind::RelationJoin(_) => PreparationRecordDomain::Mixed,
    };
    IndexPreparationPacket {
        header: IndexPreparationHeader {
            packet_index,
            identity: IndexFragmentIdentity {
                index_id: definition.index_id,
                packet_index,
            },
            reduction_key: IndexReductionKey::new(definition.index_id, packet_index),
            proof_kind: PreparationProofKind::ReadOnlyShared,
            locality: PreparationLocalityProof {
                observation_scope: crate::validation::engine::InvariantObservationKind::Committed,
                record_domain,
                partition_scope: PreparationPartitionScope::AllObserved,
                invariant_group_scope: InvariantGroupSet::empty(),
                read_set_approximation: PreparationReadSetApproximation::FullObservedScan,
                write_exclusion: PreparationWriteExclusionClass::PublicationExcluded,
            },
        },
        definition,
    }
}
