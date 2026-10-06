use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::PreparationLocalityProof;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::{AspectFieldPatch, EntityReference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImportFragmentKind {
    EntityCreate,
    RelationCreate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportFragmentIdentity {
    pub(crate) partition_id: PartitionId,
    pub(crate) kind_id: KindId,
    pub(crate) fragment_kind: ImportFragmentKind,
    pub(crate) packet_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImportStagedRow {
    Entity {
        fields: AspectFieldPatch,
    },
    Relation {
        client_key: Option<ClientKey>,
        source: EntityReference,
        target: EntityReference,
        fields: AspectFieldPatch,
    },
}

impl ImportStagedRow {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::Entity { fields } => fields.owned_allocation_capacity_bytes(),
            Self::Relation {
                client_key,
                source,
                target,
                fields,
            } => {
                let reference_bytes = |reference: &EntityReference| match reference {
                    EntityReference::Existing(_) => 0,
                    EntityReference::Created(created) => {
                        created.client_key.owned_allocation_capacity_bytes()
                    }
                };
                client_key
                    .as_ref()
                    .map_or(0, ClientKey::owned_allocation_capacity_bytes)
                    .saturating_add(reference_bytes(source))
                    .saturating_add(reference_bytes(target))
                    .saturating_add(fields.owned_allocation_capacity_bytes())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportStagingHeader {
    pub(crate) packet_index_floor: usize,
    pub(crate) identity: ImportFragmentIdentity,
    pub(crate) proof_kind: PreparationProofKind,
    pub(crate) locality: PreparationLocalityProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportStagingPacket {
    pub(crate) header: ImportStagingHeader,
    pub(crate) rows: Vec<ImportStagedRow>,
}
