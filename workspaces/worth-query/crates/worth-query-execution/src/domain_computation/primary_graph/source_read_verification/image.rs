//! An extent keeps the Native phase that actually supplied its rows.
use super::*;
use crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef;
use worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity;

pub(super) enum VerificationImage<'a> {
    Committed(RelationalPinnedRead),
    Applied {
        pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        authorization: &'a SnapshotHandle,
    },
}
impl VerificationImage<'_> {
    pub(super) fn before(&self) -> &SnapshotHandle {
        match self {
            Self::Committed(pin) => pin.snapshot(),
            Self::Applied { authorization, .. } => authorization,
        }
    }
    pub(super) fn applied(
        &self,
    ) -> Option<&RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>> {
        match self {
            Self::Committed(_) => None,
            Self::Applied { pin, .. } => Some(pin),
        }
    }
    pub(super) fn origin(&self) -> ProjectionReadOriginRef<'_> {
        match self {
            Self::Committed(pin) => pin.snapshot().into(),
            Self::Applied { pin, .. } => {
                ProjectionReadOriginRef::Applied(pin.candidate_identity().qualification())
            }
        }
    }
}
