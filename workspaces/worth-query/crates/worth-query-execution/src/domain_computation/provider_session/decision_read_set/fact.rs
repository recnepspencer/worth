use std::sync::Arc;
use worth_execution::ExecutionAllocationDenial;

use worth_query_installation::facade::WorthQueryDecisionFactKind;

use crate::domain_computation::provider_session::{
    WorthQueryProviderSessionView, WorthQuerySessionBinding,
};
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryDecisionFactRequest {
    family_identity: Arc<str>,
    locator: WorthQueryDecisionFactLocator,
}

impl WorthQueryDecisionFactRequest {
    pub fn new(
        family_identity: impl Into<Arc<str>>,
        locator: WorthQueryDecisionFactLocator,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        let family_identity = family_identity.into();
        if family_identity.trim().is_empty() || family_identity.trim() != family_identity.as_ref() {
            return Err(WorthQueryDecisionReadSetFailure::new(
                WorthQueryDecisionReadSetDenialKind::InvalidRequest,
                "decision fact family must be non-empty canonical text",
            ));
        }
        Ok(Self {
            family_identity,
            locator,
        })
    }

    pub fn family_identity(&self) -> &str {
        &self.family_identity
    }

    pub fn kind(&self) -> &WorthQueryDecisionFactKind {
        self.locator.kind()
    }

    pub fn locator(&self) -> &WorthQueryDecisionFactLocator {
        &self.locator
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryDecisionFactLocator {
    kind: WorthQueryDecisionFactKind,
    identity: Arc<str>,
}

impl WorthQueryDecisionFactLocator {
    pub fn observed(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(WorthQueryDecisionFactKind::ObservedValue, identity)
    }

    pub fn absence(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(WorthQueryDecisionFactKind::AbsenceOrNonMembership, identity)
    }

    pub fn predicate(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(WorthQueryDecisionFactKind::PredicateOrComparison, identity)
    }

    pub fn ordering(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(WorthQueryDecisionFactKind::OrderingOrExtremum, identity)
    }

    pub fn cardinality(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(
            WorthQueryDecisionFactKind::CardinalityUniquenessOrOwnership,
            identity,
        )
    }

    pub fn traversal(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(
            WorthQueryDecisionFactKind::TraversalFrontierOrPath,
            identity,
        )
    }

    pub fn access_product(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(
            WorthQueryDecisionFactKind::AccessProductCoverageOrMembership,
            identity,
        )
    }

    pub fn artifact_projection(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(
            WorthQueryDecisionFactKind::ArtifactSemanticProjection,
            identity,
        )
    }

    pub fn structural_proof(
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::new(WorthQueryDecisionFactKind::DomainStructuralProof, identity)
    }

    pub fn kind(&self) -> &WorthQueryDecisionFactKind {
        &self.kind
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    fn new(
        kind: WorthQueryDecisionFactKind,
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        let identity = identity.into();
        if identity.trim().is_empty() || identity.trim() != identity.as_ref() {
            return Err(WorthQueryDecisionReadSetFailure::new(
                WorthQueryDecisionReadSetDenialKind::InvalidRequest,
                "decision fact locator must be non-empty canonical text",
            ));
        }
        Ok(Self { kind, identity })
    }
}

pub struct WorthQueryDecisionFactAdmission {
    request: WorthQueryDecisionFactRequest,
    binding_identity: Arc<str>,
}

impl WorthQueryDecisionFactAdmission {
    pub(crate) fn new(
        request: WorthQueryDecisionFactRequest,
        binding: &WorthQuerySessionBinding,
    ) -> Self {
        Self {
            request,
            binding_identity: binding.canonical_identity().into(),
        }
    }

    pub fn observe(
        self,
        physical_version_evidence: impl Into<Arc<str>>,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        let physical_version_evidence = physical_version_evidence.into();
        if physical_version_evidence.trim().is_empty()
            || physical_version_evidence.trim() != physical_version_evidence.as_ref()
        {
            return Err(WorthQueryDecisionReadSetFailure::new(
                WorthQueryDecisionReadSetDenialKind::InvalidProviderEvidence,
                "provider fact version evidence must be non-empty canonical text",
            ));
        }
        Ok(WorthQueryDecisionFactEvidence {
            identity: Arc::clone(&physical_version_evidence),
            request: self.request,
            binding_identity: self.binding_identity,
            physical_version_evidence,
        })
    }
}

pub struct WorthQueryDecisionFactEvidence {
    identity: Arc<str>,
    request: WorthQueryDecisionFactRequest,
    binding_identity: Arc<str>,
    physical_version_evidence: Arc<str>,
}

impl WorthQueryDecisionFactEvidence {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) fn belongs_to(
        &self,
        binding: &WorthQuerySessionBinding,
        request: &WorthQueryDecisionFactRequest,
    ) -> bool {
        self.binding_identity.as_ref() == binding.canonical_identity() && &self.request == request
    }

    pub(crate) fn view(&self) -> WorthQueryDecisionFactEvidenceView<'_> {
        WorthQueryDecisionFactEvidenceView { evidence: self }
    }
}

#[derive(Clone, Copy)]
pub struct WorthQueryDecisionFactRequestView<'a> {
    request: &'a WorthQueryDecisionFactRequest,
}

impl<'a> WorthQueryDecisionFactRequestView<'a> {
    pub(crate) fn new(request: &'a WorthQueryDecisionFactRequest) -> Self {
        Self { request }
    }

    pub fn family_identity(self) -> &'a str {
        self.request.family_identity()
    }

    pub fn kind(self) -> &'a WorthQueryDecisionFactKind {
        self.request.kind()
    }

    pub fn locator(self) -> &'a WorthQueryDecisionFactLocator {
        self.request.locator()
    }
}

#[derive(Clone, Copy)]
pub struct WorthQueryDecisionFactEvidenceView<'a> {
    evidence: &'a WorthQueryDecisionFactEvidence,
}

impl<'a> WorthQueryDecisionFactEvidenceView<'a> {
    pub fn family_identity(self) -> &'a str {
        self.evidence.request.family_identity()
    }

    pub fn kind(self) -> &'a WorthQueryDecisionFactKind {
        self.evidence.request.kind()
    }

    pub fn locator(self) -> &'a WorthQueryDecisionFactLocator {
        self.evidence.request.locator()
    }

    pub fn physical_version_evidence(self) -> &'a str {
        &self.evidence.physical_version_evidence
    }
}

pub struct WorthQueryDecisionFactComparisonAdmission {
    request: WorthQueryDecisionFactRequest,
    binding_identity: Arc<str>,
    observed_physical_version: Arc<str>,
}

impl WorthQueryDecisionFactComparisonAdmission {
    pub(crate) fn new(evidence: &WorthQueryDecisionFactEvidence) -> Self {
        Self {
            request: evidence.request.clone(),
            binding_identity: Arc::clone(&evidence.binding_identity),
            observed_physical_version: Arc::clone(&evidence.physical_version_evidence),
        }
    }

    pub fn observe_current_version(
        self,
        current_physical_version: impl Into<Arc<str>>,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        let current_physical_version = current_physical_version.into();
        if current_physical_version.trim().is_empty()
            || current_physical_version.trim() != current_physical_version.as_ref()
        {
            return Err(WorthQueryDecisionReadSetFailure::new(
                WorthQueryDecisionReadSetDenialKind::InvalidProviderEvidence,
                "current provider fact version must be non-empty canonical text",
            ));
        }
        Ok(WorthQueryDecisionFactComparisonEvidence {
            request: self.request,
            binding_identity: self.binding_identity,
            observed_physical_version: self.observed_physical_version,
            current_physical_version,
        })
    }
}

pub struct WorthQueryDecisionFactComparisonEvidence {
    request: WorthQueryDecisionFactRequest,
    binding_identity: Arc<str>,
    observed_physical_version: Arc<str>,
    current_physical_version: Arc<str>,
}

impl WorthQueryDecisionFactComparisonEvidence {
    pub(crate) fn belongs_to(&self, evidence: &WorthQueryDecisionFactEvidence) -> bool {
        self.request == evidence.request
            && self.binding_identity == evidence.binding_identity
            && self.observed_physical_version == evidence.physical_version_evidence
    }

    pub(crate) fn is_fresh(&self) -> bool {
        self.current_physical_version == self.observed_physical_version
    }
}

pub trait WorthQueryDecisionFactProvider: Send + Sync + 'static {
    fn observe_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure>;

    fn compare_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDecisionReadSetDenialKind {
    InvalidRequest,
    UndeclaredFamily,
    FamilyKindMismatch,
    DecisionFactsUnsupported,
    InvalidProviderEvidence,
    EvidenceSubstitution,
    IncompleteRequiredFamilies,
    IncompleteRequiredFacts,
    DecisionFactBudgetExceeded,
    AllocationDenied,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ProviderRejected,
    ProviderPanicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionReadSetFailure {
    kind: WorthQueryDecisionReadSetDenialKind,
    detail: Arc<str>,
    allocation: Option<ExecutionAllocationDenial>,
}

impl WorthQueryDecisionReadSetFailure {
    pub fn new(kind: WorthQueryDecisionReadSetDenialKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            detail: detail.into(),
            allocation: None,
        }
    }

    pub fn kind(&self) -> WorthQueryDecisionReadSetDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Preserve the physical owner's original kind and checked byte quote.
    pub fn allocation_denied(denial: ExecutionAllocationDenial) -> Self {
        Self {
            kind: WorthQueryDecisionReadSetDenialKind::AllocationDenied,
            detail: denial.to_string().into(),
            allocation: Some(denial),
        }
    }

    pub fn allocation_denial(&self) -> Option<&ExecutionAllocationDenial> {
        self.allocation.as_ref()
    }
}
