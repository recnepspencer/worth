use crate::contracts::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::contracts::authority::commit::preparation::proofs::locality::PreparationLocalityProof;
pub mod reduction_key;
use crate::contracts::indexes::data::definition::DerivedIndexDefinition;
pub(crate) use reduction_key::IndexReductionKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexFragmentIdentity {
    pub(crate) index_id: crate::contracts::indexes::data::definition::DerivedIndexId,
    pub(crate) packet_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexPreparationHeader {
    pub(crate) packet_index: usize,
    pub(crate) identity: IndexFragmentIdentity,
    pub(crate) reduction_key: IndexReductionKey,
    pub(crate) proof_kind: PreparationProofKind,
    pub(crate) locality: PreparationLocalityProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPreparationPacket {
    pub(crate) header: IndexPreparationHeader,
    pub(crate) definition: DerivedIndexDefinition,
}
