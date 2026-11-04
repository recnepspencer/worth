//! Stable cascade admission failures; the publication context carries exact records.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthoredRetirementRefusal {
    OwnedPublicationRequiresCascade {
        publication: crate::history::data::CommitId,
        owner: crate::identity::data::EntityId,
    },
    PublicationOwnerUnavailable {
        publication: crate::history::data::CommitId,
        owner: crate::identity::data::EntityId,
    },
    MissingCoupledOwner,
    ExtraCoupledOwner,
    ForeignCoupledOwner,
    RepeatedCoupledOwner,
    MissingMember,
    MissingPublication,
    ExtraEntity,
    UnrelatedPublication,
    RepeatedTarget,
    DifferentSourcePublication,
    InvalidAuthoredAnchor,
    CorruptPublication,
    IncompleteRestoration,
}
