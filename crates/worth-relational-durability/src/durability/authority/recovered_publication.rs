//! Native owns the concrete original effect and every admitted root/pin.
use super::{ReadmittedRelationalAssociatedEffect, RefusedRelationalRecordedComponent};
use contracts::branch::basis_denial::RelationalBranchBasisDenial;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisOwnerPort as _,
};
use worth_relational_branch::internal_port::history::retention::lease::RelationalBranchRetentionLease;
use worth_relational_contracts::internal_port as contracts;

pub mod retirement;
pub mod retirement_gate_doors_for_test;
pub use retirement::{
    RelationalRecoveredPublicationRetirement, RetiredRelationalRecoveredPublication,
};
#[doc(hidden)]
pub use retirement_gate_doors_for_test::RelationalRetentionGateTrialForTest;
pub mod successor;
pub use successor::RelationalRecordedSuccessorRefusal;
pub mod subject;

/// Only successful source/body/root admission constructs this concrete payload.
/// Its named proof phase is distinct from live Performed and CommitResult.
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
    ) -> Result<
        worth_relational_branch::internal_port::history::commit::identity::RelationalCommitIdentity,
        RelationalRecordedSuccessorRefusal,
    > {
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
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
    ForeignRecoveryRuntime,
    OwnerUnavailable,
    SubjectMismatch,
    MissingOriginalPredecessor,
    AdmissionImage(
        contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
    ),
    PendingSettlement,
    MissingSettlementEvidence,
    Component(Box<RefusedRelationalRecordedComponent>),
    Retention(RelationalBranchBasisDenial),
    IncompleteCustody,
}

#[derive(Debug)]
pub struct PartialRecoveredPublication {
    pub effect: ReadmittedRelationalAssociatedEffect,
    pub predecessor: Option<AdmittedRelationalBranchBasis>,
    pub predecessor_lease: Option<RelationalBranchRetentionLease>,
    pub successor: Option<AdmittedRelationalBranchBasis>,
    pub successor_lease: Option<RelationalBranchRetentionLease>,
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
    pub fn refuse(
        self,
        cause: RelationalRecoveredPublicationAdmissionCause,
    ) -> RefusedRelationalPublicationReadmission {
        RefusedRelationalPublicationReadmission {
            cause,
            custody: self,
        }
    }
    pub fn finish(
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
