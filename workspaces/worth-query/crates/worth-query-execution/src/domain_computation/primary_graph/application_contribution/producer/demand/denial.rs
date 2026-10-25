pub(super) mod cleanup_custody;
mod diagnostic_copy;
mod invariant_refusal;
mod kind;
mod producer_value;
mod publication_stale;
pub use kind::{WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandRecoveryPosture};
pub(in crate::domain_computation::primary_graph) use publication_stale::ProducerPublicationStale;

/// A refusal to select, admit, or advance an output demand, with the subject it
/// names (usually the output family or producer).
///
/// Match on [`kind`](Self::kind) and check
/// [`recovery_posture`](Self::recovery_posture) before retrying; the subject is
/// for diagnostics.
#[derive(Debug)]
#[must_use = "a producer refusal can own native cleanup; retain or transfer it"]
pub struct WorthQueryOutputDemandDenial {
    pub(super) kind: Box<WorthQueryOutputDemandDenialKind>,
    evidence: Option<Box<DemandDenialEvidence>>,
    cleanup: Option<cleanup_custody::DemandCleanupCustody>,
    pub(super) subject: std::borrow::Cow<'static, str>,
    pub(super) domain_reason: Option<&'static str>,
    domain_value: Option<Box<super::super::WorthQueryProducerDenialValue>>,
    pub(in crate::domain_computation::primary_graph) readmission_failure: Option<&'static str>,
    pub(in crate::domain_computation::primary_graph) recovery_posture:
        WorthQueryOutputDemandRecoveryPosture,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DemandDenialEvidence {
    custom_invariant: Option<crate::domain_computation::WorthQueryCustomInvariantDenial>,
    producer_requirement: Option<
        crate::domain_computation::primary_graph::WorthQueryGeneratedOutputInvariantAdmissionDenial,
    >,
    publication_stale: Option<ProducerPublicationStale>,
    publication_companion_stop: Option<worth_relational::facade::mvcc::CompanionPreflightStop>,
    producer_admission_stop: Option<worth_relational::facade::mvcc::CompanionPreflightStop>,
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
    capacity_stop: Option<
        crate::domain_computation::primary_graph::invariant_projection::VerificationCapacityStop,
    >,
}

impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn with_managed_cleanup(
        mut self,
        cleanup: Box<
            crate::domain_computation::primary_graph::WorthQueryApplicationCleanupRequired,
        >,
    ) -> Self {
        self.cleanup = Some(cleanup_custody::DemandCleanupCustody::new(cleanup));
        self
    }
    /// Transfers the unique native cleanup and business evidence. Diagnostic clones own none.
    pub fn take_managed_cleanup(
        &mut self,
    ) -> Option<Box<crate::domain_computation::primary_graph::WorthQueryApplicationCleanupRequired>>
    {
        match self.cleanup.take()?.into_required() {
            Ok(required) => Some(required),
            Err(custody) => {
                self.cleanup = Some(custody);
                None
            }
        }
    }
    pub(super) fn managed_cleanup_failure(
        &mut self,
    ) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure> {
        self.cleanup.as_mut().and_then(|custody| custody.failure())
    }
    pub(super) fn release_managed_cleanup(
        &mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<crate::domain_computation::WorthQueryDirectRunCleanupReceipt>, Self> {
        match self.cleanup.as_mut().map(|custody| custody.release(phase)) {
            Some(Err(denial)) => Err(denial),
            Some(Ok(receipt)) => Ok(receipt),
            None => Ok(None),
        }
    }

    pub(super) fn close_read_cleanup<S: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<(), super::WorthQueryOutputReadCloseDenial> {
        if let Some(cleanup) = self.cleanup.as_mut() {
            cleanup.close_read(phase, &mut |recovery| {
                runtime.close_read_product_publication_recovery_in_advancement(phase, recovery)
            })?;
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn with_uncommitted(
        mut self,
        outcome: crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted,
    ) -> Self {
        self.cleanup = Some(cleanup_custody::DemandCleanupCustody::from_outcome(outcome));
        self
    }
    pub(super) fn has_read_cleanup(&self) -> bool {
        self.cleanup.is_some()
    }
    pub(in crate::domain_computation::primary_graph) fn with_read_cleanup(
        mut self,
        cleanup: super::WorthQueryOutputReadCleanup,
    ) -> Self {
        self.cleanup = Some(cleanup.custody);
        self
    }
    /// Transfers actual native producer outcome/cleanup custody; diagnostic clones own none.
    pub fn take_read_cleanup(&mut self) -> Option<super::WorthQueryOutputReadCleanup> {
        self.cleanup
            .take()
            .map(super::WorthQueryOutputReadCleanup::from_custody)
    }

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
    /// The bounded resource that refused verification or its meter, with
    /// that owner's typed amounts. The denial kind and posture are unchanged.
    pub(in crate::domain_computation::primary_graph) fn with_capacity_stop(
        mut self,
        stop: Option<
            crate::domain_computation::primary_graph::invariant_projection::VerificationCapacityStop,
        >,
    ) -> Self {
        if let Some(stop) = stop {
            self.evidence_mut().capacity_stop = Some(stop);
        }
        self
    }

    pub fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind.as_ref().clone()
    }

    /// Original capacity evidence from output verification or producer
    /// preparation/readmission. It may overlap `producer_admission_stop`;
    /// presence alone does not prove a precommit check ran. The original meter
    /// quote, scratch-size overflow or allocator refusal remains distinct.
    /// Diagnostic data grants no admission, currentness, retry or cleanup.
    pub fn output_verification_capacity_stop(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::VerificationCapacityStop> {
        self.evidence
            .as_ref()
            .and_then(|evidence| evidence.capacity_stop)
    }

    /// The original publication-companion resource stop from a producer commit.
    /// Its typed amounts are unchanged; an overflow carries no invented quote.
    /// Other denial paths return `None`. This is diagnostic data, not cleanup or
    /// permission to retry, and it does not change the denial kind or posture.
    pub const fn publication_companion_stop(
        &self,
    ) -> Option<worth_relational::facade::mvcc::CompanionPreflightStop> {
        match &self.evidence {
            Some(evidence) => evidence.publication_companion_stop,
            None => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_publication_companion_stop(
        mut self,
        stop: worth_relational::facade::mvcc::CompanionPreflightStop,
    ) -> Self {
        self.evidence_mut().publication_companion_stop = Some(stop);
        self
    }

    /// The original native stop during producer preparation or readmission.
    /// Unlike `publication_companion_stop`, this can describe topology, source,
    /// position or registration rather than resource exhaustion. It grants no
    /// retry or cleanup authority. `None` does not identify another cause.
    pub const fn producer_admission_stop(
        &self,
    ) -> Option<worth_relational::facade::mvcc::CompanionPreflightStop> {
        match &self.evidence {
            Some(evidence) => evidence.producer_admission_stop,
            None => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_producer_admission_stop(
        mut self,
        stop: Option<worth_relational::facade::mvcc::CompanionPreflightStop>,
    ) -> Self {
        if let Some(stop) = stop {
            self.evidence_mut().producer_admission_stop = Some(stop);
        }
        self
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
        let recovery_posture = kind.default_recovery_posture();
        Self {
            kind: Box::new(kind),
            evidence: None,
            cleanup: None,
            domain_reason: None,
            domain_value: None,
            readmission_failure: None,
            subject: subject.into(),
            recovery_posture,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn producer_domain_denied(
        subject: impl Into<std::borrow::Cow<'static, str>>,
        reason: Option<&'static str>,
        value: super::super::WorthQueryProducerDenialValue,
    ) -> Self {
        let mut denial = Self::new(
            WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
            subject,
        );
        denial.domain_reason = reason;
        denial.domain_value = Some(Box::new(value));
        denial
    }

    pub(in crate::domain_computation::primary_graph) fn product_selection(
        denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        use crate::basis::WorthQueryProductBranchAdmissionDenial as Branch;
        use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
        let kind = match denial {
            Branch::Stopped(Err(Control::Cancelled)) => WorthQueryOutputDemandDenialKind::Cancelled,
            Branch::Stopped(Err(Control::TimedOut)) => WorthQueryOutputDemandDenialKind::TimedOut,
            Branch::Stopped(Ok(cause)) => WorthQueryOutputDemandDenialKind::SourceQueryExecution(
                crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(cause)),
            Branch::Refused(refused) => WorthQueryOutputDemandDenialKind::ProductSelection(refused),
        };
        Self::new(kind, subject)
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

impl worth_execution::ChargedBytes for WorthQueryOutputDemandDenial {
    fn additional_charged_bytes(&self) -> u64 {
        use WorthQueryOutputDemandDenialKind as Kind;
        if self.evidence.is_some()
            || self.cleanup.is_some()
            || matches!(
                self.kind.as_ref(),
                Kind::ManagedCleanupRequired(_)
                    | Kind::ProductDelivery(_)
                    | Kind::CorrespondenceDelivery(_)
                    | Kind::BridgeConditional(_)
            )
        {
            return u64::MAX;
        }
        let text = match &self.subject {
            std::borrow::Cow::Borrowed(_) => 0,
            std::borrow::Cow::Owned(text) => text.capacity() as u64,
        };
        (std::mem::size_of_val(self.kind.as_ref()) as u64)
            .saturating_add(text)
            .saturating_add(self.producer_denial_value_charged_bytes())
    }
}
