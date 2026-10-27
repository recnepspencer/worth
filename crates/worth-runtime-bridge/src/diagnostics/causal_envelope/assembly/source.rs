//! Closed resolution modes preserve the legacy route and genuine Native boundary.
use super::super::{
    retained_mapping, BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial,
    BridgeCausalEnvelopeDenialKind, BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
    BridgeCausalEvidenceReference,
};
use super::native_evidence::NativeEvidence;
use crate::diagnostics::BridgeDiagnosticsFacade;
use crate::identity::BridgeIdentityEvidence;

pub(super) enum SourceContext<'a> {
    LegacyDiagnostics(&'a BridgeDiagnosticsFacade),
    Native(&'a NativeEvidence<'a>),
}

pub(super) enum ResolvedBinding {
    LegacyRetained(BridgeIdentityEvidence),
    NativeRetained,
    External,
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
