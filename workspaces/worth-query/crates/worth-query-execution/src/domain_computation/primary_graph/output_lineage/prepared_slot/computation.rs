//! Single-assignment custody of a prepared record's computation completion.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::PriorAbsence;

pub(in crate::domain_computation::primary_graph::output_lineage) enum PreparedComputationCustody {
    Unassigned,
    Assigned(SealedComputationRetention),
    Transferred,
}

impl PreparedComputationCustody {
    pub(super) fn assign(&mut self, result: SealedComputationRetention) {
        assert!(
            matches!(self, Self::Unassigned),
            "a prepared slot retains completion once"
        );
        *self = Self::Assigned(result);
    }
    pub(super) fn take(&mut self) -> SealedComputationRetention {
        assert!(
            !matches!(self, Self::Transferred),
            "a slot transfers its result once"
        );
        match std::mem::replace(self, Self::Transferred) {
            Self::Unassigned => SealedComputationRetention::Absent(PriorAbsence::NotProduced),
            Self::Assigned(result) => result,
            Self::Transferred => unreachable!(),
        }
    }
}
