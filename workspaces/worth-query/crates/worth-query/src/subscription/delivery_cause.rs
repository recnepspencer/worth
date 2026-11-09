use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::evidence_identities::delivery_cause_identity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionDeliveryCauseKind {
    RelationalPatch,
    FreshnessOnly,
    WindowEntry,
    WindowExit,
    Deadline,
    PreviousValueTransition,
    AsyncCompletion,
    AsyncDeniedCompletion,
    AsyncRetry,
    AsyncRevalidation,
    MixedCause,
}

impl QuerySubscriptionDeliveryCauseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RelationalPatch => "relational_patch",
            Self::FreshnessOnly => "freshness_only",
            Self::WindowEntry => "window_entry",
            Self::WindowExit => "window_exit",
            Self::Deadline => "deadline",
            Self::PreviousValueTransition => "previous_value_transition",
            Self::AsyncCompletion => "async_completion",
            Self::AsyncDeniedCompletion => "async_denied_completion",
            Self::AsyncRetry => "async_retry",
            Self::AsyncRevalidation => "async_revalidation",
            Self::MixedCause => "mixed_cause",
        }
    }

    pub fn has_relational_patch(self) -> bool {
        matches!(self, Self::RelationalPatch)
    }

    pub fn requires_previous_value(self) -> bool {
        matches!(self, Self::PreviousValueTransition)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDeliveryCause {
    kind: QuerySubscriptionDeliveryCauseKind,
    evidence_identity: WorthQueryEvidenceIdentity,
    delivery_cause_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionDeliveryCause {
    pub fn relational_patch(evidence_identity: &WorthQueryEvidenceIdentity) -> Self {
        Self::new(
            QuerySubscriptionDeliveryCauseKind::RelationalPatch,
            evidence_identity,
        )
    }

    pub fn time_only(
        kind: QuerySubscriptionDeliveryCauseKind,
        evidence_label: impl AsRef<str>,
    ) -> Result<Self, IdentityDerivationDenial> {
        assert!(
            !kind.has_relational_patch(),
            "time-only delivery causes cannot reuse the relational patch kind"
        );
        Ok(Self::new(
            kind,
            &delivery_cause_evidence_label_identity(evidence_label.as_ref())?,
        ))
    }
    #[cfg(test)]
    pub(crate) fn classified(
        kind: QuerySubscriptionDeliveryCauseKind,
        evidence_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(kind, evidence_identity))
    }

    fn new(
        kind: QuerySubscriptionDeliveryCauseKind,
        evidence_identity: &WorthQueryEvidenceIdentity,
    ) -> Self {
        let evidence_identity = evidence_identity.clone();
        let delivery_cause_identity = delivery_cause_identity(kind, &evidence_identity);
        Self {
            kind,
            evidence_identity,
            delivery_cause_identity,
        }
    }

    pub fn kind(&self) -> QuerySubscriptionDeliveryCauseKind {
        self.kind
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }

    pub fn delivery_cause_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_cause_identity
    }

    pub fn has_relational_patch(&self) -> bool {
        self.kind.has_relational_patch()
    }
}

pub(crate) fn delivery_cause_evidence_label_identity(
    label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(super::evidence_identities::delivery_cause_evidence_label_identity(label)?)
}
