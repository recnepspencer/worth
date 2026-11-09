use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainInstallationDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    DuplicateMarkerType,
    DuplicatePackageIdentity,
    ConflictingDomainOwner,
    ConflictingInvariant,
    ConflictingGraphReadOperation,
    AmbiguousGraphReadRelationScope,
    ConflictingDeclarationFamily,
    InvariantLoweringFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainInstallationDenial {
    kind: WorthQueryDomainInstallationDenialKind,
    subject: String,
}

impl WorthQueryDomainInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryDomainInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryDomainInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryDomainInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "domain installation denied during {}: {}",
            installation_denial_stage(self.kind),
            self.subject
        )
    }
}

impl std::error::Error for WorthQueryDomainInstallationDenial {}

fn installation_denial_stage(kind: WorthQueryDomainInstallationDenialKind) -> &'static str {
    match kind {
        WorthQueryDomainInstallationDenialKind::EvidenceIdentityDerivationDenied(_) => {
            "identity materialization"
        }
        WorthQueryDomainInstallationDenialKind::DuplicateMarkerType => "marker admission",
        WorthQueryDomainInstallationDenialKind::DuplicatePackageIdentity => "package admission",
        WorthQueryDomainInstallationDenialKind::ConflictingDomainOwner => "owner admission",
        WorthQueryDomainInstallationDenialKind::ConflictingInvariant => "invariant lowering",
        WorthQueryDomainInstallationDenialKind::ConflictingGraphReadOperation => {
            "graph-read operation lowering"
        }
        WorthQueryDomainInstallationDenialKind::AmbiguousGraphReadRelationScope => {
            "graph-read relation admission"
        }
        WorthQueryDomainInstallationDenialKind::ConflictingDeclarationFamily => {
            "declaration-family lowering"
        }
        WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed => "invariant compilation",
    }
}

impl From<IdentityDerivationDenial> for WorthQueryDomainInstallationDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::new(
            WorthQueryDomainInstallationDenialKind::EvidenceIdentityDerivationDenied(cause),
            format!("{:?}", cause.scope),
        )
    }
}
