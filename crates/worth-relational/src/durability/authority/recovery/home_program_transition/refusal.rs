use crate::durability::authority::RefusedRelationalRecordedComponent;
use crate::durability::data::{DurabilityError, RelationalRecoveryStop};

#[derive(Debug)]
pub enum RelationalHomeProgramTransitionSourceRefusal {
    Recovery(RelationalRecoveryStop),
    Predecessor(RefusedRelationalRecordedComponent),
    PredecessorMissing,
    PredecessorSchemaMismatch,
    CurrentHeadMissing,
    DurableSourceMissing {
        commit: crate::history::data::CommitId,
    },
    PrefixLength {
        expected: usize,
        actual: usize,
    },
    PrefixAssociation {
        position: crate::publication::patch::data::PatchStreamPosition,
    },
    PrefixParent {
        commit: crate::history::data::CommitId,
    },
    PrefixBranch {
        commit: crate::history::data::CommitId,
    },
    PrefixHead,
    Backing(worth_execution::ExecutionRequestArrayDenial),
    CapacityOverflow,
    UnboundHome,
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    AssignmentCount,
    Contracts(Box<crate::runtime::RelationalHomeProgramContractDenial>),
}
impl From<RelationalRecoveryStop> for RelationalHomeProgramTransitionSourceRefusal {
    fn from(cause: RelationalRecoveryStop) -> Self {
        Self::Recovery(cause)
    }
}
impl From<DurabilityError> for RelationalHomeProgramTransitionSourceRefusal {
    fn from(cause: DurabilityError) -> Self {
        Self::Recovery(cause.into())
    }
}
