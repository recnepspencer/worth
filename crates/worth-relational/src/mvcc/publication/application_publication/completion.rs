//! Complete installed correspondence supplied with concrete publication admission.
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::EntityReference;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequiredOutputPublicationMeaning {
    pub implementation: super::RequiredOutputProducingImplementation,
    pub family: String,
    pub source_scope: EntityId,
    pub source_partition: [u8; 32],
    pub source_epoch: [u8; 32],
}

impl RequiredOutputPublicationMeaning {
    // A new build succeeds the same authored route. Exact readmission still
    // compares the complete original meaning, including its implementation.
    pub(crate) fn same_route(&self, other: &Self) -> bool {
        self.family == other.family
            && self.source_scope == other.source_scope
            && self.source_partition == other.source_partition
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RequiredOutputRolePosture {
    Create,
    Preserve,
    Retire,
}

#[derive(Clone, Debug)]
pub struct RequiredOutputRoleTarget {
    pub role: String,
    pub entity_name: String,
    pub kind: KindId,
    pub posture: RequiredOutputRolePosture,
    pub target: EntityReference,
}

#[derive(Debug)]
pub enum RequiredOutputPublication {
    Birth {
        admission: worth_proof::AdmittedRequiredOutputPublication,
        absence: Option<super::RelationalRequiredOutputRouteAbsence>,
        meaning: RequiredOutputPublicationMeaning,
        roles: Vec<RequiredOutputRoleTarget>,
    },
    Preserve {
        admission: worth_proof::AdmittedRequiredOutputSuccession,
        predecessor: super::RelationalRequiredOutputPublication,
        meaning: RequiredOutputPublicationMeaning,
        roles: Vec<RequiredOutputRoleTarget>,
    },
}

impl RequiredOutputPublication {
    pub(crate) fn running(&self) -> &worth_proof::AdmittedRequiredOutputPublication {
        match self {
            Self::Birth { admission, .. } => admission,
            Self::Preserve { admission, .. } => admission.running(),
        }
    }
}
