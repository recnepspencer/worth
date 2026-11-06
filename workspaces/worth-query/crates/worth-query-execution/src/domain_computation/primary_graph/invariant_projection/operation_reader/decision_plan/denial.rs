//! Typed refusal of undeclared or foreign decision targets.
/// Why a decision read in an operation projection was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantDecisionPlanDenialKind {
    /// The operation does not declare this target as a decision read.
    UndeclaredDecisionTarget,
    /// The identity came from a different projection authority.
    ForeignIdentity,
    /// The field is not installed.
    FieldNotInstalled,
}

/// Refusal of a decision read in an operation projection.
///
/// [`Self::kind`] says why and [`Self::subject`] names the target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantDecisionPlanDenial {
    kind: WorthQueryInvariantDecisionPlanDenialKind,
    subject: String,
}

impl WorthQueryInvariantDecisionPlanDenial {
    pub const fn kind(&self) -> WorthQueryInvariantDecisionPlanDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub(super) fn new(
        kind: WorthQueryInvariantDecisionPlanDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryInvariantDecisionPlanDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invariant decision plan denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryInvariantDecisionPlanDenial {}

impl worth_execution::ChargedBytes for WorthQueryInvariantDecisionPlanDenial {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { kind: _, subject } = self;
        subject.capacity() as u64
    }
}
