mod kind;
pub use kind::{WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandRecoveryPosture};

/// A refusal to select, admit, or advance an output demand, with the subject it
/// names (usually the output family or producer).
///
/// Match on [`kind`](Self::kind) and check
/// [`recovery_posture`](Self::recovery_posture) before retrying; the subject is
/// for diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandDenial {
    pub(super) kind: WorthQueryOutputDemandDenialKind,
    evidence: Option<Box<DemandDenialEvidence>>,
    pub(super) subject: std::borrow::Cow<'static, str>,
    pub(super) domain_reason: Option<&'static str>,
    pub(in crate::domain_computation::primary_graph) readmission_failure: Option<&'static str>,
    pub(in crate::domain_computation::primary_graph) recovery_posture:
        WorthQueryOutputDemandRecoveryPosture,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DemandDenialEvidence {
    commit_denial_kind:
        Option<crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind>,
    commit_execution: Option<(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    )>,
    requested_output:
        Option<crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead>,
}

impl WorthQueryOutputDemandDenial {
    fn evidence_mut(&mut self) -> &mut DemandDenialEvidence {
        self.evidence.get_or_insert_with(Default::default)
    }

    pub(in crate::domain_computation::primary_graph) fn with_requested_output(
        mut self,
        requested: crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead,
    ) -> Self {
        self.evidence_mut().requested_output = Some(requested);
        self
    }

    pub(in crate::domain_computation::primary_graph) fn take_requested_output(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead>
    {
        self.evidence
            .as_mut()
            .and_then(|evidence| evidence.requested_output.take())
    }
    pub fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind.clone()
    }

    /// The original pre-effect commit refusal, when this demand ran a producer.
    pub const fn commit_denial_kind(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind>
    {
        match &self.evidence {
            Some(evidence) => evidence.commit_denial_kind,
            None => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_commit_denial_kind(
        mut self,
        kind: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
    ) -> Self {
        self.evidence_mut().commit_denial_kind = Some(kind);
        self
    }

    /// The original typed execution cause and its publication-refusal stage.
    pub fn commit_execution_denial(
        &self,
    ) -> Option<(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    )> {
        self.evidence
            .as_ref()
            .and_then(|evidence| evidence.commit_execution)
    }

    pub(in crate::domain_computation::primary_graph) fn with_commit_execution_denial(
        mut self,
        evidence: Option<(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
            Result<
                crate::domain_computation::WorthQueryProviderSessionDenialKind,
                crate::domain_computation::WorthQueryProviderSessionControlStopKind,
            >,
        )>,
    ) -> Self {
        if let Some(evidence) = evidence {
            self.evidence_mut().commit_execution = Some(evidence);
        }
        self
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
            kind: kind.clone(),
            evidence: None,
            domain_reason: None,
            readmission_failure: None,
            subject: subject.into(),
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

impl WorthQueryOutputDemandDenial {
    /// A typed refusal before the caller pass enters its request.
    pub fn request_admission(
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
            "request admission",
        )
    }
}
