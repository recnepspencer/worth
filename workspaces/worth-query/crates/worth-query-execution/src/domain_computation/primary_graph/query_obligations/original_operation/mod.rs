//! Immutable original meaning; native World and output owners retain effect custody.
use worth_foundational::facade::ProgramSupportObligationIdentity;
pub(super) mod codec;
mod fold;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QueryOriginalOperationPosition {
    Initial,
    Resolving,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QueryOriginalOperationData {
    pub revision: [u8; 32],
    pub program: String,
    pub action: String,
    pub binding: String,
    pub branch: String,
    /// The publication owner encodes input, key, preconditions and exact source DATA.
    pub description: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOriginalOperationObligation {
    pub identity: ProgramSupportObligationIdentity,
    pub data: QueryOriginalOperationData,
    pub position: QueryOriginalOperationPosition,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryOriginalOperationRecord {
    Retain(QueryOriginalOperationObligation),
    Advance {
        identity: ProgramSupportObligationIdentity,
        position: QueryOriginalOperationPosition,
    },
    Discharge {
        identity: ProgramSupportObligationIdentity,
    },
}
