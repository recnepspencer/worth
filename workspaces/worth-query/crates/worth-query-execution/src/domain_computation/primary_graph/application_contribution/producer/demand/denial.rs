use super::{WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandRecoveryPosture};

/// A refusal to select, admit, or advance an output demand, with the subject it
/// names (usually the output family or producer).
///
/// Match on [`kind`](Self::kind) and check
/// [`recovery_posture`](Self::recovery_posture) before retrying; the subject is
/// for diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandDenial {
    pub(super) kind: WorthQueryOutputDemandDenialKind,
    pub(super) subject: std::borrow::Cow<'static, str>,
    pub(super) domain_reason: Option<&'static str>,
    pub(in crate::domain_computation::primary_graph) readmission_failure: Option<&'static str>,
    pub(in crate::domain_computation::primary_graph) requested_output:
        Option<crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead>,
    pub(in crate::domain_computation::primary_graph) recovery_posture:
        WorthQueryOutputDemandRecoveryPosture,
}

impl WorthQueryOutputDemandDenial {
    pub const fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Domain-owned diagnostic words, when the rejected producer supplies them.
    /// These words do not replace the typed denial kind or confer authority.
    pub const fn domain_reason(&self) -> Option<&'static str> {
        self.domain_reason
    }

    /// Why an exact failed native read could not join retained required work.
    /// Diagnostic only; this does not change the denial kind or retry posture.
    pub const fn readmission_failure(&self) -> Option<&'static str> {
        self.readmission_failure
    }

    pub const fn recovery_posture(&self) -> WorthQueryOutputDemandRecoveryPosture {
        self.recovery_posture
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryOutputDemandDenialKind,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            domain_reason: None,
            readmission_failure: None,
            requested_output: None,
            recovery_posture: kind.default_recovery_posture(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn producer_domain_denied(
        subject: impl Into<std::borrow::Cow<'static, str>>,
        reason: Option<&'static str>,
    ) -> Self {
        let mut denial = Self::new(
            WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
            subject,
        );
        denial.domain_reason = reason;
        denial
    }

    pub(in crate::domain_computation::primary_graph) fn product_selection(
        denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::ProductSelection(denial),
            subject,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn with_recovery_posture(
        mut self,
        recovery_posture: WorthQueryOutputDemandRecoveryPosture,
    ) -> Self {
        self.recovery_posture = recovery_posture;
        self
    }
}
