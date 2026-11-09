use crate::evidence_identity::WorthQueryEvidenceIdentity;
#[cfg(test)]
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSupportPosture;

use super::evidence_identities::runtime_downstream_resume_posture_identity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeDownstreamResumePostureKind {
    RuntimeBackedAdmitted,
    MissingBasisDenied,
    StaleBasisDenied,
    SupportGateDeferred,
    SupportGateDenied,
    DurableDeferredDebt,
}

impl WorthQueryRuntimeDownstreamResumePostureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeBackedAdmitted => "runtime-backed-admitted",
            Self::MissingBasisDenied => "missing-basis-denied",
            Self::StaleBasisDenied => "stale-basis-denied",
            Self::SupportGateDeferred => "support-gate-deferred",
            Self::SupportGateDenied => "support-gate-denied",
            Self::DurableDeferredDebt => "durable-deferred-debt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeDownstreamResumePosture {
    kind: WorthQueryRuntimeDownstreamResumePostureKind,
    required_basis_identity: Option<WorthQueryEvidenceIdentity>,
    support_posture: WorthQueryLowerRuntimeSupportPosture,
    support_identity: WorthQueryEvidenceIdentity,
    posture_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeDownstreamResumePosture {
    pub(crate) fn new(
        kind: WorthQueryRuntimeDownstreamResumePostureKind,
        required_basis_identity: Option<WorthQueryEvidenceIdentity>,
        support_posture: WorthQueryLowerRuntimeSupportPosture,
        support_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        let posture_identity = runtime_downstream_resume_posture_identity(
            kind,
            required_basis_identity.as_ref(),
            support_posture,
            &support_identity,
        );
        Self {
            kind,
            required_basis_identity,
            support_posture,
            support_identity,
            posture_identity,
        }
    }

    pub fn kind(&self) -> WorthQueryRuntimeDownstreamResumePostureKind {
        self.kind
    }

    pub fn required_basis_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.required_basis_identity.as_ref()
    }

    pub fn required_basis_for_reporting(&self) -> Option<&str> {
        self.required_basis_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn support_posture(&self) -> WorthQueryLowerRuntimeSupportPosture {
        self.support_posture
    }

    pub fn support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_identity
    }

    pub fn support_for_reporting(&self) -> &str {
        self.support_identity.as_str()
    }

    pub fn posture_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.posture_identity
    }

    pub fn posture_for_reporting(&self) -> &str {
        self.posture_identity.as_str()
    }
}

pub(crate) fn aggregate_support_posture(
    postures: impl IntoIterator<Item = WorthQueryLowerRuntimeSupportPosture>,
) -> WorthQueryLowerRuntimeSupportPosture {
    postures
        .into_iter()
        .max_by_key(|posture| support_posture_rank(*posture))
        .unwrap_or(WorthQueryLowerRuntimeSupportPosture::Deferred)
}

pub(crate) fn support_gate_resume_kind(
    posture: WorthQueryLowerRuntimeSupportPosture,
) -> WorthQueryRuntimeDownstreamResumePostureKind {
    match posture {
        WorthQueryLowerRuntimeSupportPosture::Admitted => {
            WorthQueryRuntimeDownstreamResumePostureKind::RuntimeBackedAdmitted
        }
        WorthQueryLowerRuntimeSupportPosture::Deferred => {
            WorthQueryRuntimeDownstreamResumePostureKind::SupportGateDeferred
        }
        WorthQueryLowerRuntimeSupportPosture::CompatibilityDebt
        | WorthQueryLowerRuntimeSupportPosture::SeamEliminated
        | WorthQueryLowerRuntimeSupportPosture::Forbidden => {
            WorthQueryRuntimeDownstreamResumePostureKind::SupportGateDenied
        }
    }
}

fn support_posture_rank(posture: WorthQueryLowerRuntimeSupportPosture) -> u8 {
    match posture {
        WorthQueryLowerRuntimeSupportPosture::Admitted => 0,
        WorthQueryLowerRuntimeSupportPosture::CompatibilityDebt => 1,
        WorthQueryLowerRuntimeSupportPosture::SeamEliminated => 2,
        WorthQueryLowerRuntimeSupportPosture::Deferred => 3,
        WorthQueryLowerRuntimeSupportPosture::Forbidden => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower_runtime_routing::{
        worth_query_lower_runtime_support_matrix, WorthQueryLowerRuntimeSeamKey,
    };
    use crate::runtime::evidence_identities::lower_runtime_support_row_identity;

    #[test]
    fn aggregate_support_posture_uses_strictest_row() {
        assert_eq!(
            aggregate_support_posture([
                WorthQueryLowerRuntimeSupportPosture::Deferred,
                WorthQueryLowerRuntimeSupportPosture::Admitted,
                WorthQueryLowerRuntimeSupportPosture::Forbidden,
            ]),
            WorthQueryLowerRuntimeSupportPosture::Forbidden
        );
    }

    #[test]
    fn support_gate_resume_kind_fails_closed_for_non_admitted_runtime_support() {
        assert_eq!(
            support_gate_resume_kind(WorthQueryLowerRuntimeSupportPosture::Deferred),
            WorthQueryRuntimeDownstreamResumePostureKind::SupportGateDeferred
        );
        assert_eq!(
            support_gate_resume_kind(WorthQueryLowerRuntimeSupportPosture::CompatibilityDebt),
            WorthQueryRuntimeDownstreamResumePostureKind::SupportGateDenied
        );
        assert_eq!(
            support_gate_resume_kind(WorthQueryLowerRuntimeSupportPosture::Forbidden),
            WorthQueryRuntimeDownstreamResumePostureKind::SupportGateDenied
        );
    }

    #[test]
    fn resume_posture_carries_typed_support_identity() -> Result<(), IdentityDerivationDenial> {
        let matrix = worth_query_lower_runtime_support_matrix();
        let support = matrix
            .support_for(WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromSubscriptionEvidence)
            .expect("support row must exist");
        let support_identity = lower_runtime_support_row_identity(support)?;
        let posture = WorthQueryRuntimeDownstreamResumePosture::new(
            WorthQueryRuntimeDownstreamResumePostureKind::SupportGateDeferred,
            None,
            support.posture(),
            support_identity,
        );
        assert_eq!(posture.support_identity(), posture.support_identity());

        Ok(())
    }
}
