//! Bank-facing read-only view of Query's committed dispatch outbox.

use worth_query_host::facade::primary_graph::{
    WorthQueryCommittedDispatchOutboxObservation,
    WorthQueryCommittedDispatchOutboxReadDenial as QueryDenial,
};

use crate::{BankCommitReceipt, BankIdentityRuntime};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankCommittedDispatchOutboxObservation {
    query: WorthQueryCommittedDispatchOutboxObservation,
}

impl BankCommittedDispatchOutboxObservation {
    pub fn correlation(&self) -> &[u8; 32] {
        self.query.record().correlation().bytes()
    }

    pub fn correlation_family(&self) -> &str {
        self.query.record().correlation_family().as_str()
    }

    pub fn effect(&self) -> &str {
        self.query.record().effect()
    }

    pub const fn protocol_identity(&self) -> &worth_foundational::facade::BoundaryProtocolIdentity {
        self.query.record().protocol_identity()
    }

    pub const fn protocol_version(&self) -> worth_foundational::facade::BoundaryProtocolVersion {
        self.query.record().protocol_version()
    }

    pub const fn maximum_payload_bytes(&self) -> u64 {
        self.query.record().maximum_payload_bytes()
    }

    pub fn payload(&self) -> &[u8] {
        self.query.record().payload()
    }

    pub const fn outcome_identity(&self) -> u64 {
        self.query.record().outcome_identity()
    }

    pub const fn commit_id(&self) -> u64 {
        self.query.commit_reference().commit_id.0
    }

    pub const fn commit_version(&self) -> u64 {
        self.query.commit_reference().version_id.0
    }

    pub fn commit_branch(&self) -> &str {
        &self.query.commit_reference().branch_id.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommittedDispatchOutboxReadDenial {
    ForeignRuntime,
    Missing,
    WrongRecordKind,
    NotAuthoritative,
    ExactCommitUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    Malformed,
    CommitMismatch,
    RecordMismatch,
}

impl BankIdentityRuntime {
    /// Reads a committed effect from a fresh Query-provider owner view.
    pub fn observe_committed_dispatch_outbox(
        &self,
        receipt: &BankCommitReceipt,
    ) -> Result<Option<BankCommittedDispatchOutboxObservation>, BankCommittedDispatchOutboxReadDenial>
    {
        receipt
            .recovery_evidence()
            .observe_dispatch_outbox(self.application_runtime())
            .map(|observation| {
                observation.map(|query| BankCommittedDispatchOutboxObservation { query })
            })
            .map_err(Into::into)
    }
}

impl From<QueryDenial> for BankCommittedDispatchOutboxReadDenial {
    fn from(denial: QueryDenial) -> Self {
        match denial {
            QueryDenial::ForeignRuntime => Self::ForeignRuntime,
            QueryDenial::Missing => Self::Missing,
            QueryDenial::WrongRecordKind => Self::WrongRecordKind,
            QueryDenial::NotAuthoritative => Self::NotAuthoritative,
            QueryDenial::ExactCommitUnavailable => Self::ExactCommitUnavailable,
            QueryDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            QueryDenial::SnapshotIdentityExhausted => Self::SnapshotIdentityExhausted,
            QueryDenial::Malformed => Self::Malformed,
            QueryDenial::CommitMismatch => Self::CommitMismatch,
            QueryDenial::RecordMismatch => Self::RecordMismatch,
        }
    }
}
