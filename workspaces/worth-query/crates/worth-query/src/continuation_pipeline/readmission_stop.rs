use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryContinuationExecutionReadmissionStopKind {
    StaleBasis,
    AsyncRequestDrift,
    ReplayDrift,
    PolicyRemaskDrift,
    PreviewCrossedResidue,
    StaleCompletion,
    BasisMismatch,
    LowerBindingMismatch,
    AuthorityMismatch,
}

impl WorthQueryContinuationExecutionReadmissionStopKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::StaleBasis => "stale-basis",
            Self::AsyncRequestDrift => "async-request-drift",
            Self::ReplayDrift => "replay-drift",
            Self::PolicyRemaskDrift => "policy-remask-drift",
            Self::PreviewCrossedResidue => "preview-crossed-residue",
            Self::StaleCompletion => "stale-completion",
            Self::BasisMismatch => "basis-mismatch",
            Self::LowerBindingMismatch => "lower-binding-mismatch",
            Self::AuthorityMismatch => "authority-mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryContinuationExecutionReadmissionNextAction {
    RefreshBasis,
    RebindContext,
    CheckPolicySupport,
    UseExplicitHandoff,
    InspectProofLane,
}

impl WorthQueryContinuationExecutionReadmissionNextAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::RefreshBasis => "refresh-basis",
            Self::RebindContext => "rebind-context",
            Self::CheckPolicySupport => "check-policy-support",
            Self::UseExplicitHandoff => "use-explicit-handoff",
            Self::InspectProofLane => "inspect-proof-lane",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryContinuationExecutionReadmissionCounters {
    planning_attempts: u64,
    lower_runtime_attempts: u64,
    execution_attempts: u64,
}

impl WorthQueryContinuationExecutionReadmissionCounters {
    const fn denied_before_later_work() -> Self {
        Self {
            planning_attempts: 0,
            lower_runtime_attempts: 0,
            execution_attempts: 0,
        }
    }

    pub const fn planning_attempts(self) -> u64 {
        self.planning_attempts
    }

    pub const fn lower_runtime_attempts(self) -> u64 {
        self.lower_runtime_attempts
    }

    pub const fn execution_attempts(self) -> u64 {
        self.execution_attempts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuationExecutionReadmissionStop {
    kind: WorthQueryContinuationExecutionReadmissionStopKind,
    next_action: WorthQueryContinuationExecutionReadmissionNextAction,
    counters: WorthQueryContinuationExecutionReadmissionCounters,
    reason: String,
    stop_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryContinuationExecutionReadmissionStop {
    pub(crate) fn new(
        kind: WorthQueryContinuationExecutionReadmissionStopKind,
        reason: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let next_action = next_action_for(kind);
        let counters =
            WorthQueryContinuationExecutionReadmissionCounters::denied_before_later_work();
        let reason = reason.into();
        let stop_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::ContinuationExecutionReadmissionEvidence,
        )
        .field_shape(WorthQueryEvidenceTag::new("outcome"), "stopped")
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("next_action"),
            next_action.as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("planning_attempts"),
            counters.planning_attempts().to_string(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("lower_runtime_attempts"),
            counters.lower_runtime_attempts().to_string(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_attempts"),
            counters.execution_attempts().to_string(),
        )
        .seal()?;
        Ok(Self {
            kind,
            next_action,
            counters,
            reason,
            stop_identity,
        })
    }

    pub const fn kind(&self) -> WorthQueryContinuationExecutionReadmissionStopKind {
        self.kind
    }

    pub const fn next_action(&self) -> WorthQueryContinuationExecutionReadmissionNextAction {
        self.next_action
    }

    pub const fn counters(&self) -> WorthQueryContinuationExecutionReadmissionCounters {
        self.counters
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn stop_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.stop_identity
    }
}

const fn next_action_for(
    kind: WorthQueryContinuationExecutionReadmissionStopKind,
) -> WorthQueryContinuationExecutionReadmissionNextAction {
    use WorthQueryContinuationExecutionReadmissionNextAction as Action;
    use WorthQueryContinuationExecutionReadmissionStopKind as Kind;

    match kind {
        Kind::StaleBasis | Kind::ReplayDrift | Kind::StaleCompletion | Kind::BasisMismatch => {
            Action::RefreshBasis
        }
        Kind::AsyncRequestDrift => Action::RebindContext,
        Kind::PolicyRemaskDrift => Action::CheckPolicySupport,
        Kind::PreviewCrossedResidue => Action::UseExplicitHandoff,
        Kind::LowerBindingMismatch | Kind::AuthorityMismatch => Action::InspectProofLane,
    }
}
