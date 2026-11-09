//! A real producer can participate without claiming publication ownership.
use super::*;

/// Untrusted posture selectors with the installed producer's concrete admission.
/// Native admits these against C before applying any participant commands to D.
#[derive(Debug)]
pub enum RelationalCoupledProducerPosture {
    Publication(RequiredOutputPublication),
    Ordinary {
        admission: worth_proof::AdmittedRequiredOutputPublication,
        absence: RelationalRequiredOutputRouteAbsence,
        meaning: RequiredOutputPublicationMeaning,
        roles: Vec<RequiredOutputRoleTarget>,
    },
}
impl RelationalCoupledProducerPosture {
    pub fn producing_meaning(&self) -> Option<&RequiredOutputPublicationMeaning> {
        match self {
            Self::Publication(publication) => publication.producing_meaning(),
            Self::Ordinary { meaning, .. } => Some(meaning),
        }
    }
    pub(crate) fn lineage_attribution(&self) -> Option<(&str, [u8; 32])> {
        match self {
            Self::Publication(publication) => publication.lineage_attribution(),
            Self::Ordinary { admission, .. } => {
                Some((admission.producer(), *admission.dependency_identity()))
            }
        }
    }
    pub(crate) fn admitted_by(
        &self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
    ) -> bool {
        match self {
            Self::Publication(publication) => publication.admitted_by(installed),
            Self::Ordinary { admission, .. } => {
                installed.authority.admits_required_output(admission)
            }
        }
    }
    pub(crate) fn owned(&self) -> Option<&RequiredOutputPublication> {
        match self {
            Self::Publication(publication) => Some(publication),
            Self::Ordinary { .. } => None,
        }
    }
    pub(super) fn roles_mut(&mut self) -> &mut Vec<RequiredOutputRoleTarget> {
        match self {
            Self::Publication(
                RequiredOutputPublication::Birth { roles, .. }
                | RequiredOutputPublication::Preserve { roles, .. },
            )
            | Self::Ordinary { roles, .. } => roles,
            _ => unreachable!("Native admitted a producer participant"),
        }
    }
}
