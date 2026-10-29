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
    AuthoredRetirement {
        admission: worth_proof::AdmittedAuthoredOutputRetirement,
        predecessor: super::RelationalRequiredOutputPublication,
        roles: Vec<RequiredOutputRoleTarget>,
    },
}

impl RequiredOutputPublication {
    pub(crate) fn admitted_by(
        &self,
        installed: &super::RelationalInstalledApplicationPublicationAuthority,
    ) -> bool {
        match self {
            Self::Birth { admission, .. } => installed.authority.admits_required_output(admission),
            Self::Preserve {
                admission,
                predecessor,
                ..
            } => installed
                .authority
                .admits_required_output_succession(admission, predecessor.authority()),
            Self::AuthoredRetirement {
                admission,
                predecessor,
                ..
            } => installed
                .authority
                .admits_authored_output_retirement(admission, predecessor.authority()),
        }
    }

    pub(crate) fn lineage_attribution(&self) -> (&str, [u8; 32]) {
        match self {
            Self::Birth { admission, .. } => {
                (admission.producer(), *admission.dependency_identity())
            }
            Self::Preserve { admission, .. } => (
                admission.running().producer(),
                *admission.running().dependency_identity(),
            ),
            Self::AuthoredRetirement { predecessor, .. } => {
                // Provenance of the retired lineage, not a claim that the
                // authored mutation ran this producer again.
                let origin = predecessor
                    .envelope
                    .required_output_origin
                    .as_ref()
                    .expect("Native custody retains its canonical publication");
                (&origin.producer, origin.dependency_identity)
            }
        }
    }
}
