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
    InvalidManagedIndexDefinition,
    ManagedIndexBindingMeaningChanged,
    ManagedIndexSourceUnsupported,
    ManagedIndexParentUnsupported,
    ManagedIndexUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryInstallationDenial {
    kind: WorthQueryApplicationQueryInstallationDenialKind,
    subject: String,
    canonical_denial: Option<CanonicalDigestDerivationDenial>,
}

impl WorthQueryApplicationQueryInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationQueryInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            canonical_denial: None,
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub const fn canonical_denial(&self) -> Option<&CanonicalDigestDerivationDenial> {
        self.canonical_denial.as_ref()
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
    WorthQueryApplicationQueryInstallationDenial {
        kind,
        subject: subject.to_owned(),
        canonical_denial: Some(denial),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_obligation_budget_refusals_preserve_requested_and_maximum_counts() {
        for cause in [
            CanonicalDigestDerivationDenial::EntryLimitExceeded {
                actual: 17,
                maximum: 16,
            },
            CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
                attempted: 33,
                maximum: 32,
            },
        ] {
            let denial = graph_obligation_denial(
                "query",
                WorthQueryGraphObligationInstallationDenial::Canonical(cause),
            );
            assert_eq!(denial.canonical_denial(), Some(&cause));
            assert_eq!(denial.subject(), "query");
        }
    }
}
