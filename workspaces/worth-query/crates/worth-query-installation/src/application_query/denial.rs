#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryInstallationDenialKind {
    BindingIdentityCollision,
    BindingMeaningChanged,
    BindingPrincipalMeaningChanged,
    BindingQueryMeaningChanged,
    BindingQueryNotInstalled,
    BindingResultLimitIsZero,
    BindingScopeMeaningChanged,
    InvalidBindingIdentity,
    QueryNotInstalled,
    QueryMeaningChanged,
    AuthorizationNotInstalled,
    LiveEffectNotInstalled,
    LiveScopeIdentityNotInstalled,
    LiveTargetIdentityNotInstalled,
    RelationNotInstalled,
    ForeignRuntime,
    StaleGeneration,
    SchemaMeaningChanged,
    PackageIdentityChanged,
    AuthorityMismatch,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
    InvalidGraphObligationContract,
    InvalidInspectionDefinition,
    InspectionBindingMeaningChanged,
    InspectionUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryInstallationDenial {
    kind: WorthQueryApplicationQueryInstallationDenialKind,
    subject: String,
}

impl WorthQueryApplicationQueryInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationQueryInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryApplicationQueryInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application query installation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationQueryInstallationDenial {}

use crate::graph_obligation::WorthQueryGraphObligationInstallationDenial;
use worth_foundational::facade::CanonicalDigestDerivationDenial;

pub(super) fn graph_obligation_denial(
    subject: &str,
    denial: WorthQueryGraphObligationInstallationDenial,
) -> WorthQueryApplicationQueryInstallationDenial {
    let kind = match denial {
        WorthQueryGraphObligationInstallationDenial::InvalidContract => {
            WorthQueryApplicationQueryInstallationDenialKind::InvalidGraphObligationContract
        }
        WorthQueryGraphObligationInstallationDenial::Canonical(denial) => {
            return canonical_work_denial(subject, denial)
        }
    };
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}

pub(super) fn canonical_work_denial(
    subject: &str,
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryApplicationQueryInstallationDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryApplicationQueryInstallationDenialKind::CanonicalEntryBudgetExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryApplicationQueryInstallationDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        _ => WorthQueryApplicationQueryInstallationDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}
