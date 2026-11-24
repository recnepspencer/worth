use crate::authority::commit::preparation::packets::index::IndexReductionKey;
use crate::authority::commit::preparation::packets::index::{
    IndexFragmentIdentity, IndexPreparationHeader, IndexPreparationPacket,
};
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationLocalityProof, PreparationPartitionScope, PreparationReadSetApproximation,
    PreparationRecordDomain, PreparationWriteExclusionClass,
};
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::runtime::state::subsystems::indexing::IndexingSubsystem;
use crate::validation::data::groups::InvariantGroupSet;

mod checked;
pub(super) use checked::{plan_index_packets_checked, PlannedIndexPackets};

pub(super) fn planned_index_definitions(
    indexes: &IndexingSubsystem,
    index_ids: &[DerivedIndexId],
) -> (Vec<DerivedIndexDefinition>, Vec<DerivedIndexId>) {
    let mut definitions = Vec::new();
    let mut missing_indexes = Vec::new();

    for index_id in index_ids {
        if let Some(definition) = indexes.definition(*index_id) {
            definitions.push(definition.as_ref().clone());
        } else {
            missing_indexes.push(*index_id);
        }
    }

    (definitions, missing_indexes)
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
                observation_scope:
                    crate::validation::data::observation_kind::InvariantObservationKind::Committed,
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
