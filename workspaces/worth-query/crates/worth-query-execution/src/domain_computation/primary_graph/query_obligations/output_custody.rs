//! Required work is durable data; installed executors and World issue live custody.
use worth_foundational::facade::{CanonicalDigestId, DurableCompositeCommitIdentity};

pub(super) mod codec;
pub(super) mod fold;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOutputSourceObligation {
    pub commit: DurableCompositeCommitIdentity,
    pub revision: CanonicalDigestId,
    pub root: String,
    pub branch: String,
    pub lifecycle: u64,
    pub generation: u64,
    pub scope: (u32, u64, u32),
    pub key: [u8; 32],
    pub intent: String,
    pub discovery_recipe: Option<Vec<u8>>,
    pub bindings: Option<Vec<([u8; 32], (u32, u64, u32))>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryOutputSourceRecord {
    Bind {
        commit: DurableCompositeCommitIdentity,
        sources: Vec<([u8; 32], (u32, u64, u32))>,
    },
    Retain(QueryOutputSourceObligation),
    Discharge {
        commit: DurableCompositeCommitIdentity,
    },
}

/// The position is the shared journal's acquire position, never a runtime ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOutputDemandObligation {
    pub acquire_position: std::num::NonZeroU64,
    pub revision: CanonicalDigestId,
    pub family: String,
    pub producer: String,
    pub query: [u8; 32],
    pub branch: String,
    pub basis: DurableCompositeCommitIdentity,
    pub root: [u8; 16],
    /// Encoded admitted parameter values, re-admitted by the installed query.
    pub parameters: Vec<u8>,
    pub scope: Vec<u8>,
    pub performed_sources: Vec<DurableCompositeCommitIdentity>,
    pub prerequisites: Vec<std::num::NonZeroU64>,
    pub checkpoint_prerequisites: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryOutputDemandRecord {
    DomainDenied {
        acquire_position: std::num::NonZeroU64,
        failure: Vec<u8>,
    },
    Retain(QueryOutputDemandObligation),
    Links {
        acquire_position: std::num::NonZeroU64,
        performed_sources: Vec<DurableCompositeCommitIdentity>,
        prerequisites: Vec<std::num::NonZeroU64>,
        checkpoint_prerequisites: Vec<Vec<u8>>,
    },
    Discharge {
        acquire_position: std::num::NonZeroU64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct OutputDemandSubject {
    revision: CanonicalDigestId,
    family: String,
    producer: String,
    query: [u8; 32],
    basis: DurableCompositeCommitIdentity,
    branch: String,
    root: [u8; 16],
    parameters: Vec<u8>,
    scope: Vec<u8>,
}
impl QueryOutputDemandObligation {
    pub(super) fn subject(&self) -> OutputDemandSubject {
        OutputDemandSubject {
            revision: self.revision,
            family: self.family.clone(),
            producer: self.producer.clone(),
            query: self.query,
            basis: self.basis,
            branch: self.branch.clone(),
            root: self.root,
            parameters: self.parameters.clone(),
            scope: self.scope.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryAcceptedOutputRecord {
    pub commit: DurableCompositeCommitIdentity,
    pub branch: String,
    pub producer: String,
    pub scope: (u32, u64, u32),
    pub partition: [u8; 32],
    pub payload: Vec<u8>,
}
