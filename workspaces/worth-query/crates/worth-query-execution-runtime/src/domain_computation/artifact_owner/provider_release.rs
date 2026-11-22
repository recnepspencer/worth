#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactProviderDisposalDisposition {
    Completed,
    Panicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactProviderDestructorDisposition {
    Completed,
    Panicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactProviderReleaseEvidence {
    disposal: WorthQueryArtifactProviderDisposalDisposition,
    destructor: WorthQueryArtifactProviderDestructorDisposition,
}

impl WorthQueryArtifactProviderReleaseEvidence {
    pub(super) const fn new(
        disposal: WorthQueryArtifactProviderDisposalDisposition,
        destructor: WorthQueryArtifactProviderDestructorDisposition,
    ) -> Self {
        Self {
            disposal,
            destructor,
        }
    }

    pub const fn disposal(self) -> WorthQueryArtifactProviderDisposalDisposition {
        self.disposal
    }

    pub const fn destructor(self) -> WorthQueryArtifactProviderDestructorDisposition {
        self.destructor
    }

    pub const fn recovery_required(self) -> bool {
        matches!(
            self.disposal,
            WorthQueryArtifactProviderDisposalDisposition::Panicked
        ) || matches!(
            self.destructor,
            WorthQueryArtifactProviderDestructorDisposition::Panicked
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactProviderReleasePosture {
    Retained,
    Pending,
    Complete(WorthQueryArtifactProviderReleaseEvidence),
    RecoveryRequired(WorthQueryArtifactProviderReleaseEvidence),
}

impl WorthQueryArtifactProviderReleasePosture {
    pub(super) const fn from_evidence(evidence: WorthQueryArtifactProviderReleaseEvidence) -> Self {
        if evidence.recovery_required() {
            Self::RecoveryRequired(evidence)
        } else {
            Self::Complete(evidence)
        }
    }

    pub const fn recovery_required(self) -> bool {
        matches!(self, Self::RecoveryRequired(_))
    }
}
