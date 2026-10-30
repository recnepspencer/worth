//! Closed resolution modes preserve the legacy route and genuine Native boundary.
use super::super::{
    retained_mapping, BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial,
    BridgeCausalEnvelopeDenialKind, BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
    BridgeCausalEvidenceReference,
};
use super::native_evidence::NativeEvidence;
use super::{
    progress::BridgeCausalEnvelopeAssemblyProgress, BridgeCausalEnvelopeAssemblyRequest,
    BridgeCausalExplanationEnvelope,
};
use crate::diagnostics::BridgeDiagnosticsFacade;
use crate::identity::BridgeIdentityEvidence;

/// Closed source strategies share sequencing while retaining their own truth,
/// resource admission, failure causes and descriptive output posture.
pub(super) trait CausalEnvelopeAssemblySource {
    type Stop;

    fn prepare_progress(
        &mut self,
        request: &BridgeCausalEnvelopeAssemblyRequest,
    ) -> Result<BridgeCausalEnvelopeAssemblyProgress, Self::Stop>;
    fn bind_reference(
        &mut self,
        progress: &mut BridgeCausalEnvelopeAssemblyProgress,
        reference: &BridgeCausalEvidenceReference,
    ) -> Result<(), Self::Stop>;
    fn requires_bridge_route(&self) -> bool;
    fn missing_bridge_route(
        &mut self,
        request: &BridgeCausalEnvelopeAssemblyRequest,
        progress: &BridgeCausalEnvelopeAssemblyProgress,
    ) -> Self::Stop;
    fn finish(
        self,
        request: BridgeCausalEnvelopeAssemblyRequest,
        progress: BridgeCausalEnvelopeAssemblyProgress,
    ) -> Result<BridgeCausalExplanationEnvelope, Self::Stop>;
}

pub(super) enum SourceContext<'a> {
    LegacyDiagnostics(&'a BridgeDiagnosticsFacade),
    Native(&'a NativeEvidence<'a>),
}

pub(super) enum ResolvedBinding {
    LegacyRetained(BridgeIdentityEvidence),
    NativeRetained,
    External,
}

impl CausalEnvelopeAssemblySource for SourceContext<'_> {
    type Stop = BridgeCausalEnvelopeDenial;

    fn prepare_progress(
        &mut self,
        request: &BridgeCausalEnvelopeAssemblyRequest,
    ) -> Result<BridgeCausalEnvelopeAssemblyProgress, Self::Stop> {
        if request
            .admission_summary()
            .requires_application_inspection_admission()
        {
            return Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::ApplicationInspectionAdmissionRequired,
                BridgeCausalEvidenceFamily::QueryObservation,
                BridgeCausalEvidenceOwner::Query,
                BridgeCausalEvidenceOwner::Query,
                request.request_evidence_identity().clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ));
        }
        Ok(BridgeCausalEnvelopeAssemblyProgress::for_request(
            request,
            self.is_native(),
        ))
    }

    fn bind_reference(
        &mut self,
        progress: &mut BridgeCausalEnvelopeAssemblyProgress,
        reference: &BridgeCausalEvidenceReference,
    ) -> Result<(), Self::Stop> {
        progress.bind_reference(self, reference)
    }

    fn requires_bridge_route(&self) -> bool {
        !self.is_native()
    }

    fn missing_bridge_route(
        &mut self,
        request: &BridgeCausalEnvelopeAssemblyRequest,
        progress: &BridgeCausalEnvelopeAssemblyProgress,
    ) -> Self::Stop {
        BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::MissingRequiredBridgeRouteEvidence,
            BridgeCausalEvidenceFamily::BridgeRoute,
            BridgeCausalEvidenceOwner::RuntimeBridge,
            BridgeCausalEvidenceOwner::RuntimeBridge,
            request.request_evidence_identity().clone(),
            progress.success_counters(),
        )
    }

    fn finish(
        self,
        request: BridgeCausalEnvelopeAssemblyRequest,
        progress: BridgeCausalEnvelopeAssemblyProgress,
    ) -> Result<BridgeCausalExplanationEnvelope, Self::Stop> {
        let counters = progress.success_counters();
        Ok(BridgeCausalExplanationEnvelope::new(
            request,
            progress.into_bindings(),
            counters,
        ))
    }
}

impl SourceContext<'_> {
    pub(super) fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    pub(super) fn needs_registry_lookup(&self, reference: &BridgeCausalEvidenceReference) -> bool {
        matches!(self, Self::LegacyDiagnostics(_))
            && reference.owner() == BridgeCausalEvidenceOwner::RuntimeBridge
            && reference.family() != BridgeCausalEvidenceFamily::BridgeConditionalObservation
    }

    pub(super) fn resolve(
        &self,
        reference: &BridgeCausalEvidenceReference,
        counters: BridgeCausalEnvelopeCounters,
    ) -> Result<ResolvedBinding, BridgeCausalEnvelopeDenial> {
        match self {
            Self::Native(evidence) if evidence.contains(reference) => {
                Ok(ResolvedBinding::NativeRetained)
            }
            Self::Native(_)
                if reference.family() == BridgeCausalEvidenceFamily::QueryObservation =>
            {
                Ok(ResolvedBinding::External)
            }
            Self::Native(_) => Err(native_required(reference, counters)),
            Self::LegacyDiagnostics(_)
                if reference.family()
                    == BridgeCausalEvidenceFamily::BridgeConditionalObservation =>
            {
                Err(native_required(reference, counters))
            }
            Self::LegacyDiagnostics(facade)
                if reference.owner() == BridgeCausalEvidenceOwner::RuntimeBridge =>
            {
                match retained_mapping::retained_record_evidence_identity(facade, reference)? {
                    Some(identity) => Ok(ResolvedBinding::LegacyRetained(identity)),
                    None => Err(BridgeCausalEnvelopeDenial::new(
                        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord,
                        reference.family(),
                        reference.owner(),
                        reference.family().expected_owner(),
                        reference.reference_evidence_identity().clone(),
                        counters,
                    )),
                }
            }
            Self::LegacyDiagnostics(_) => Ok(ResolvedBinding::External),
        }
    }
}

fn native_required(
    reference: &BridgeCausalEvidenceReference,
    counters: BridgeCausalEnvelopeCounters,
) -> BridgeCausalEnvelopeDenial {
    BridgeCausalEnvelopeDenial::new(
        BridgeCausalEnvelopeDenialKind::NativeObservationRequired,
        reference.family(),
        reference.owner(),
        reference.family().expected_owner(),
        reference.reference_evidence_identity().clone(),
        counters,
    )
}
