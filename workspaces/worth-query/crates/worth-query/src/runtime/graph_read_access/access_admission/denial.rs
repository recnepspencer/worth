use super::WorthQueryGraphReadAccessAdmissionPosture;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessDenialKind {
    RequiredAsyncMaterialization,
    RequiredAccessCapabilityRegistration,
    RequiredPersistentIndex,
    UnsupportedGraphIndexSupport,
}

impl WorthQueryGraphReadAccessDenialKind {
    pub const ALL: [Self; 4] = [
        Self::RequiredAsyncMaterialization,
        Self::RequiredAccessCapabilityRegistration,
        Self::RequiredPersistentIndex,
        Self::UnsupportedGraphIndexSupport,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RequiredAsyncMaterialization => "required_async_materialization",
            Self::RequiredAccessCapabilityRegistration => "required_access_capability_registration",
            Self::RequiredPersistentIndex => "required_persistent_index",
            Self::UnsupportedGraphIndexSupport => "unsupported_graph_index_support",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessDenial {
    kind: WorthQueryGraphReadAccessDenialKind,
    suggested_posture: WorthQueryGraphReadAccessAdmissionPosture,
}

impl WorthQueryGraphReadAccessDenial {
    pub fn kind(&self) -> &WorthQueryGraphReadAccessDenialKind {
        &self.kind
    }

    pub fn suggested_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.suggested_posture
    }

    pub(crate) fn required_async_materialization() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::RequiredAsyncMaterialization,
            suggested_posture:
                WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired,
        }
    }

    pub(crate) fn required_access_capability_registration() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::RequiredAccessCapabilityRegistration,
            suggested_posture:
                WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired,
        }
    }

    pub(crate) fn required_persistent_index() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::RequiredPersistentIndex,
            suggested_posture: WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired,
        }
    }

    pub(crate) fn unsupported_graph_index_support() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::UnsupportedGraphIndexSupport,
            suggested_posture: WorthQueryGraphReadAccessAdmissionPosture::Denied,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "denial:{}:{}",
            self.kind.as_str(),
            self.suggested_posture.as_str()
        )
    }
}
