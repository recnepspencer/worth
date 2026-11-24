use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::PreparationLocalityProof;
mod reduction_key;
use crate::indexes::data::DerivedIndexDefinition;
pub(crate) use reduction_key::IndexReductionKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexFragmentIdentity {
    pub(crate) index_id: crate::indexes::data::DerivedIndexId,
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
pub(crate) struct IndexPreparationPacket {
    pub(crate) header: IndexPreparationHeader,
    pub(crate) definition: DerivedIndexDefinition,
}
