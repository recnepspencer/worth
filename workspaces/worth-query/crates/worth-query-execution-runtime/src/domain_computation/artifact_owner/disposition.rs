#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactDisposition {
    Produced,
    Transferred,
    Borrowed,
    Leased,
    Replaced,
    Cancelled,
    Released,
    Disposed,
}

impl WorthQueryArtifactDisposition {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Produced => "produced",
            Self::Transferred => "transferred",
            Self::Borrowed => "borrowed",
            Self::Leased => "leased",
            Self::Replaced => "replaced",
            Self::Cancelled => "cancelled",
            Self::Released => "released",
            Self::Disposed => "disposed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDisposedArtifact {
    owner_identity: String,
    occurrence_identity: String,
    disposition: WorthQueryArtifactDisposition,
    provider_release: super::WorthQueryArtifactProviderReleasePosture,
}

impl WorthQueryDisposedArtifact {
    pub(super) fn new(
        owner_identity: String,
        occurrence_identity: String,
        disposition: WorthQueryArtifactDisposition,
        provider_release: super::WorthQueryArtifactProviderReleasePosture,
    ) -> Self {
        Self {
            owner_identity,
            occurrence_identity,
            disposition,
            provider_release,
        }
    }

    pub fn owner_identity(&self) -> &str {
        &self.owner_identity
    }

    pub fn occurrence_identity(&self) -> &str {
        &self.occurrence_identity
    }

    pub const fn disposition(&self) -> WorthQueryArtifactDisposition {
        self.disposition
    }

    pub const fn provider_release(&self) -> super::WorthQueryArtifactProviderReleasePosture {
        self.provider_release
    }
}
