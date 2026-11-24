use crate::contracts::validation::data::execution::InvariantExecutionPoint;
use crate::contracts::validation::data::groups::{InvariantGroup, InvariantGroupSet};

use super::observation::InvariantObservationKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantRequestProfile {
    ProposedState,
    CommitBoundary,
    MutationSensitive,
    GraphComposition,
    SnapshotPublication,
    CertificationBoundary,
    HarnessAudit,
}

impl InvariantRequestProfile {
    pub fn execution_point(self) -> InvariantExecutionPoint {
        match self {
            Self::ProposedState => InvariantExecutionPoint::ProposedState,
            Self::CommitBoundary => InvariantExecutionPoint::CommitBoundary,
            Self::MutationSensitive => InvariantExecutionPoint::MutationSensitive,
            Self::GraphComposition => InvariantExecutionPoint::GraphComposition,
            Self::SnapshotPublication => InvariantExecutionPoint::SnapshotPublication,
            Self::CertificationBoundary => InvariantExecutionPoint::CertificationBoundary,
            Self::HarnessAudit => InvariantExecutionPoint::HarnessAudit,
        }
    }

    pub fn consumed_groups(self) -> InvariantGroupSet {
        match self {
            Self::ProposedState => Self::CommitBoundary
                .consumed_groups()
                .union(Self::MutationSensitive.consumed_groups())
                .union(Self::SnapshotPublication.consumed_groups()),
            Self::CommitBoundary => InvariantGroupSet::of(InvariantGroup::StorageCoherence)
                .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence))
                .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::LineageIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
            Self::MutationSensitive => InvariantGroupSet::of(InvariantGroup::StorageCoherence)
                .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence))
                .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance))
                .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::LineageIntegrity)),
            Self::GraphComposition => InvariantGroupSet::of(InvariantGroup::SchemaCompliance)
                .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::LineageIntegrity)),
            Self::SnapshotPublication => InvariantGroupSet::of(InvariantGroup::VersionVisibility)
                .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
            Self::CertificationBoundary => InvariantGroupSet::of(InvariantGroup::VersionVisibility)
                .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity))
                .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
            Self::HarnessAudit => InvariantGroupSet::all(),
        }
    }

    pub(crate) const fn supports_observation(self, observation: InvariantObservationKind) -> bool {
        match self {
            Self::CommitBoundary => {
                matches!(observation, InvariantObservationKind::Committed)
            }
            Self::ProposedState
            | Self::MutationSensitive
            | Self::GraphComposition
            | Self::SnapshotPublication
            | Self::CertificationBoundary => true,
            Self::HarnessAudit => matches!(observation, InvariantObservationKind::Committed),
        }
    }
}
