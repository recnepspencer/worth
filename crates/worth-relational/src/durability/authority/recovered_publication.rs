//! Native owns the concrete original effect and every admitted root/pin.
use super::{ReadmittedRelationalAssociatedEffect, RefusedRelationalRecordedComponent};
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDenial};
use crate::history::retention::RelationalBranchRetentionLease;

mod admission;
mod retirement;
pub use retirement::{
    RelationalRecoveredPublicationRetirement, RetiredRelationalRecoveredPublication,
};
mod successor;
pub use successor::RelationalRecordedSuccessorRefusal;
mod subject;

/// Only successful source/body/root admission constructs this concrete payload.
/// Its named proof phase is distinct from live Performed and CommitResult.
///
/// Actual custody remains transportable without copying its pins:
/// ```
/// use worth_proof::RelationalPublicationReadmitted;
/// use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;
/// fn transport<T: Send + Sync + 'static>() {}
/// transport::<RelationalRecoveredPublicationCustody>();
/// transport::<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>();
/// ```
///
/// ```compile_fail,E0308
/// use worth_proof::{AuthorityMarker, RelationalPublicationReadmitted};
/// use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;
/// struct ReceiptDescription;
/// impl AuthorityMarker for ReceiptDescription {}
/// fn adopt(_: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>) {}
/// adopt(RelationalPublicationReadmitted::from_owner(ReceiptDescription));
/// ```
#[derive(Debug)]
pub struct RelationalRecoveredPublicationCustody {
    effect: ReadmittedRelationalAssociatedEffect,
    predecessor: AdmittedRelationalBranchBasis,
    predecessor_lease: RelationalBranchRetentionLease,
    successor: AdmittedRelationalBranchBasis,
    successor_lease: RelationalBranchRetentionLease,
}
impl worth_proof::AuthorityMarker for RelationalRecoveredPublicationCustody {}

impl RelationalRecoveredPublicationCustody {
    pub fn effect(&self) -> &ReadmittedRelationalAssociatedEffect {
        &self.effect
    }
    pub fn predecessor(&self) -> &AdmittedRelationalBranchBasis {
        &self.predecessor
    }
    pub fn successor(&self) -> &AdmittedRelationalBranchBasis {
        &self.successor
    }

    /// Identity of the actual admitted successor, never rebuilt by a caller
    /// from receipt fields. The caller owns backing for the copied branch name.
    pub fn readmitted_commit_identity(
        &self,
    ) -> Result<crate::history::RelationalCommitIdentity, RelationalRecordedSuccessorRefusal> {
        self.successor
            .commit_identity()
            .ok_or(RelationalRecordedSuccessorRefusal::SubjectMismatch)
    }

    /// Consume once into the next owner's retained custody. Excluded live
    /// fields are unavailable; no snapshot, result or timing is fabricated.
    pub fn into_retained_parts(
        self,
    ) -> (
        ReadmittedRelationalAssociatedEffect,
        AdmittedRelationalBranchBasis,
        RelationalBranchRetentionLease,
        AdmittedRelationalBranchBasis,
        RelationalBranchRetentionLease,
    ) {
        (
            self.effect,
            self.predecessor,
            self.predecessor_lease,
            self.successor,
            self.successor_lease,
        )
    }
}

#[derive(Debug)]
pub enum RelationalRecoveredPublicationAdmissionCause {
    Request(crate::execution::RelationalExecutionDenialCause),
    ForeignRecoveryRuntime,
    OwnerUnavailable,
    SubjectMismatch,
    MissingOriginalPredecessor,
    AdmissionImage(crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal),
    PendingSettlement,
    MissingSettlementEvidence,
    Component(Box<RefusedRelationalRecordedComponent>),
    Retention(RelationalBranchBasisDenial),
    IncompleteCustody,
}

#[derive(Debug)]
struct PartialRecoveredPublication {
    effect: ReadmittedRelationalAssociatedEffect,
    predecessor: Option<AdmittedRelationalBranchBasis>,
    predecessor_lease: Option<RelationalBranchRetentionLease>,
    successor: Option<AdmittedRelationalBranchBasis>,
    successor_lease: Option<RelationalBranchRetentionLease>,
}

#[derive(Debug)]
pub struct RefusedRelationalPublicationReadmission {
    cause: RelationalRecoveredPublicationAdmissionCause,
    custody: PartialRecoveredPublication,
}

impl RefusedRelationalPublicationReadmission {
    pub fn cause(&self) -> &RelationalRecoveredPublicationAdmissionCause {
        &self.cause
    }
    pub fn effect(&self) -> &ReadmittedRelationalAssociatedEffect {
        &self.custody.effect
    }
    pub fn retains_acquired_custody(&self) -> bool {
        self.custody.predecessor.is_some()
            || self.custody.successor.is_some()
            || self.custody.predecessor_lease.is_some()
            || self.custody.successor_lease.is_some()
            || matches!(&self.cause, RelationalRecoveredPublicationAdmissionCause::Component(cause) if cause.retains_acquired_custody())
    }
    /// Explicitly release partial new pins while retaining the original source
    /// subject/image for a fresh caller-funded request.
    pub fn into_effect(self) -> ReadmittedRelationalAssociatedEffect {
        self.custody.effect
    }
}

impl PartialRecoveredPublication {
    fn refuse(
        self,
        cause: RelationalRecoveredPublicationAdmissionCause,
    ) -> RefusedRelationalPublicationReadmission {
        RefusedRelationalPublicationReadmission {
            cause,
            custody: self,
        }
    }
    fn finish(
        self,
    ) -> Result<RelationalRecoveredPublicationCustody, RefusedRelationalPublicationReadmission>
    {
        match self {
            Self {
                effect,
                predecessor: Some(predecessor),
                predecessor_lease: Some(predecessor_lease),
                successor: Some(successor),
                successor_lease: Some(successor_lease),
            } => Ok(RelationalRecoveredPublicationCustody {
                effect,
                predecessor,
                predecessor_lease,
                successor,
                successor_lease,
            }),
            partial => {
                Err(partial.refuse(RelationalRecoveredPublicationAdmissionCause::IncompleteCustody))
            }
        }
    }
}
