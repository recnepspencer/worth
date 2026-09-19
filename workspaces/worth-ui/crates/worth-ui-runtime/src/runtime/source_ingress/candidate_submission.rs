use crate::capability::CapabilitySnapshot;
use crate::runtime::replacement::candidate::rust_authored_replacement_candidate;
use crate::runtime::source_ingress::counters::WorthUiSourceIngressCounters;
use crate::runtime::source_ingress::denial::WorthUiSourceIngressDenial;
use crate::runtime::source_ingress::ordering_receipt::WorthUiCandidateOrderingReceipt;
use crate::runtime::source_ingress::revision::WorthUiSourcePackageRevision;
use crate::runtime::source_ingress::{
    prepare_semantic_handoff, WorthUiCandidateComposition, WorthUiCandidateCompositionBasis,
    WorthUiCandidatePreparationHandoff,
};
use crate::runtime::WorthUiReplacementCandidateDenial;
use crate::runtime::WorthUiReplacementCause;
use worth_ui_dsl::{WorthUiDslCompileReport, WorthUiDslCompiler, WorthUiRustAuthoredArtifactInput};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthUiWatchedCandidateSubmission {
    composition: WorthUiCandidateComposition,
    revision: WorthUiSourcePackageRevision,
    ordering_receipt: WorthUiCandidateOrderingReceipt,
    counters: WorthUiSourceIngressCounters,
    retained_observation_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiWatchedCandidateSubmissionDenial {
    DslCompilation(Box<WorthUiDslCompileReport>),
    SourceIngress(WorthUiSourceIngressDenial),
    RuntimePreparation(Box<crate::runtime::WorthUiSemanticHandoffPreparationDenial>),
    Candidate(Box<WorthUiReplacementCandidateDenial>),
}

pub(crate) enum WorthUiAuthoredCompositionPreparationDenial {
    DslCompilation(Box<WorthUiDslCompileReport>),
    RuntimePreparation(Box<crate::runtime::WorthUiSemanticHandoffPreparationDenial>),
    Candidate(Box<WorthUiReplacementCandidateDenial>),
}

pub(crate) fn prepare_rust_authored_handoff(
    input: &WorthUiRustAuthoredArtifactInput,
    snapshot: &CapabilitySnapshot,
) -> Result<WorthUiCandidatePreparationHandoff, WorthUiAuthoredCompositionPreparationDenial> {
    let source_revision_digest = input.source_revision_digest();
    let sealed_package = WorthUiDslCompiler::compile_rust_authored(input).map_err(|denial| {
        WorthUiAuthoredCompositionPreparationDenial::DslCompilation(Box::new(denial))
    })?;
    let material = prepare_semantic_handoff(sealed_package, snapshot).map_err(|denial| {
        WorthUiAuthoredCompositionPreparationDenial::RuntimePreparation(Box::new(denial))
    })?;
    let (artifact, declaration_material, handoff) = material.into_parts();
    let candidate = rust_authored_replacement_candidate(
        artifact,
        handoff.successor_snapshot_digest(),
        WorthUiReplacementCause::rust_authored_input_change(source_revision_digest),
    )
    .map_err(|denial| WorthUiAuthoredCompositionPreparationDenial::Candidate(Box::new(denial)))?;
    Ok(
        WorthUiCandidateComposition::rust_authored(candidate, declaration_material, handoff)
            .into_preparation_handoff(),
    )
}

impl WorthUiWatchedCandidateSubmission {
    pub(crate) fn from_source_attempt(
        composition: WorthUiCandidateComposition,
        revision: WorthUiSourcePackageRevision,
        ordering_receipt: WorthUiCandidateOrderingReceipt,
        counters: WorthUiSourceIngressCounters,
        retained_observation_bytes: usize,
    ) -> Self {
        Self {
            composition,
            revision,
            ordering_receipt,
            counters,
            retained_observation_bytes,
        }
    }

    pub fn ordering_receipt(&self) -> &WorthUiCandidateOrderingReceipt {
        &self.ordering_receipt
    }

    pub fn source_revision(&self) -> &WorthUiSourcePackageRevision {
        &self.revision
    }

    pub fn counters(&self) -> WorthUiSourceIngressCounters {
        self.counters
    }

    pub(crate) const fn retained_observation_bytes(&self) -> usize {
        self.retained_observation_bytes
    }

    pub(crate) fn snapshot_digest(&self) -> u64 {
        self.composition.snapshot_digest()
    }

    pub fn composition_basis(&self) -> &WorthUiCandidateCompositionBasis {
        self.composition.basis()
    }

    pub(crate) fn authored_source_basis(&self) -> super::WorthUiAuthoredSourceBasis {
        super::WorthUiAuthoredSourceBasis::watched(
            self.revision.clone(),
            self.ordering_receipt.clone(),
            self.composition.basis().clone(),
        )
    }

    pub fn authoring_lane(&self) -> crate::runtime::WorthUiCandidateAuthoringLane {
        self.composition.authoring_lane()
    }

    pub(crate) fn candidate_snapshot_digest(&self) -> u64 {
        self.composition.snapshot_digest()
    }

    pub(crate) fn into_preparation_handoff(self) -> WorthUiCandidatePreparationHandoff {
        self.composition.into_preparation_handoff()
    }

    pub(crate) fn into_replacement_handoff(self) -> WorthUiCandidatePreparationHandoff {
        self.composition.into_preparation_handoff()
    }
}
