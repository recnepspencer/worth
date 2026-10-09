use std::collections::BTreeMap;
use std::sync::Arc;

use super::{WorthQueryLoweredProvisionalEffectProgram, WorthQueryProvisionalFailure};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryProposedFactOrigin {
    AuthoritativeBase,
    StagedReplacement,
    StagedCreation,
    StagedRetirement,
    DerivedProvisionalView,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProposedFact {
    identity: Arc<str>,
    origin: WorthQueryProposedFactOrigin,
    semantic_value: Arc<str>,
}

impl WorthQueryProposedFact {
    pub fn new(
        identity: impl Into<Arc<str>>,
        origin: WorthQueryProposedFactOrigin,
        semantic_value: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryProvisionalFailure> {
        let identity = canonical(identity)?;
        let semantic_value = canonical(semantic_value)?;
        Ok(Self {
            identity,
            origin,
            semantic_value,
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn origin(&self) -> WorthQueryProposedFactOrigin {
        self.origin
    }

    pub fn semantic_value(&self) -> &str {
        &self.semantic_value
    }
}

#[derive(Clone, Copy)]
pub struct WorthQueryProvisionalEffectProgramView<'a> {
    program: &'a WorthQueryLoweredProvisionalEffectProgram,
    generation: u64,
}

impl<'a> WorthQueryProvisionalEffectProgramView<'a> {
    pub(crate) fn new(
        program: &'a WorthQueryLoweredProvisionalEffectProgram,
        generation: u64,
    ) -> Self {
        Self {
            program,
            generation,
        }
    }

    pub fn identity(self) -> &'a str {
        self.program.identity()
    }

    pub fn steps(self) -> &'a [super::WorthQueryProvisionalEffectStep] {
        self.program.steps()
    }

    pub fn generation(self) -> u64 {
        self.generation
    }
}

pub struct WorthQueryProvisionalOverlayAdmission {
    cleanup: WorthQueryProvisionalOverlayCleanupBinding,
    binding_identity: Arc<str>,
    program_identity: Arc<str>,
    generation: u64,
}

impl WorthQueryProvisionalOverlayAdmission {
    pub(in crate::domain_computation) fn new(
        session: crate::domain_computation::provider_session::WorthQueryProviderSessionView<'_>,
        binding_identity: &str,
        program_identity: &str,
        generation: u64,
    ) -> Self {
        Self {
            cleanup: WorthQueryProvisionalOverlayCleanupBinding::from_session(session),
            binding_identity: binding_identity.into(),
            program_identity: program_identity.into(),
            generation,
        }
    }

    pub fn admit(
        self,
        physical_overlay_identity: impl Into<Arc<str>>,
        facts: impl IntoIterator<Item = WorthQueryProposedFact>,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
        let physical_overlay_identity = canonical(physical_overlay_identity)?;
        let mut facts = facts.into_iter().collect::<Vec<_>>();
        facts.sort();
        if facts
            .windows(2)
            .any(|pair| pair[0].identity == pair[1].identity)
        {
            return Err(WorthQueryProvisionalFailure::invalid_program(
                "provider returned duplicate proposed fact identities",
            ));
        }
        Ok(WorthQueryProvisionalOverlayEvidence {
            cleanup: self.cleanup,
            identity: Arc::clone(&physical_overlay_identity),
            proposed_state_identity: Arc::clone(&physical_overlay_identity),
            binding_identity: self.binding_identity,
            program_identity: self.program_identity,
            physical_overlay_identity,
            generation: self.generation,
            facts: facts.into(),
        })
    }
}

pub struct WorthQueryProvisionalOverlayEvidence {
    cleanup: WorthQueryProvisionalOverlayCleanupBinding,
    identity: Arc<str>,
    proposed_state_identity: Arc<str>,
    binding_identity: Arc<str>,
    program_identity: Arc<str>,
    physical_overlay_identity: Arc<str>,
    generation: u64,
    facts: Arc<[WorthQueryProposedFact]>,
}

pub(in crate::domain_computation) struct WorthQueryProvisionalOverlayCleanupBinding {
    affinity:
        crate::domain_computation::provider_session::WorthQueryProviderSessionAffinityIdentity,
    token_identity: Arc<str>,
    token_generation: u64,
    provider_identity: Arc<str>,
    provider_generation: u64,
    plan_identity: Arc<str>,
}

impl WorthQueryProvisionalOverlayCleanupBinding {
    fn from_session(
        session: crate::domain_computation::provider_session::WorthQueryProviderSessionView<'_>,
    ) -> Self {
        Self {
            affinity: session.affinity_identity(),
            token_identity: session.identity().into(),
            token_generation: session.generation(),
            provider_identity: session.provider_identity().into(),
            provider_generation: session.provider_generation(),
            plan_identity: session.plan_identity().into(),
        }
    }

    pub(in crate::domain_computation) const fn affinity_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionAffinityIdentity
    {
        self.affinity
    }

    pub(in crate::domain_computation::provider_session) fn token_identity(&self) -> &str {
        &self.token_identity
    }

    pub(in crate::domain_computation::provider_session) const fn token_generation(&self) -> u64 {
        self.token_generation
    }

    pub(in crate::domain_computation::provider_session) fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub(in crate::domain_computation::provider_session) const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }

    pub(in crate::domain_computation::provider_session) fn plan_identity(&self) -> &str {
        &self.plan_identity
    }
}

impl WorthQueryProvisionalOverlayEvidence {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) fn proposed_state_identity(&self) -> &str {
        &self.proposed_state_identity
    }

    pub(crate) fn facts(&self) -> &[WorthQueryProposedFact] {
        &self.facts
    }

    pub(crate) fn belongs_to(
        &self,
        binding_identity: &str,
        program_identity: &str,
        generation: u64,
    ) -> bool {
        self.binding_identity.as_ref() == binding_identity
            && self.program_identity.as_ref() == program_identity
            && self.generation == generation
    }

    pub(crate) fn matches_program(
        &self,
        program: &WorthQueryLoweredProvisionalEffectProgram,
    ) -> bool {
        let mut expected = BTreeMap::new();
        for step in program.steps() {
            let (identity, origin) = expected_proposed_fact(step.action());
            if expected
                .insert(identity, origin)
                .is_some_and(|prior| prior != origin)
            {
                return false;
            }
        }
        let mut observed = 0;
        for fact in self
            .facts
            .iter()
            .filter(|fact| fact.origin() != WorthQueryProposedFactOrigin::AuthoritativeBase)
        {
            if expected.get(fact.identity()) != Some(&fact.origin()) {
                return false;
            }
            observed += 1;
        }
        observed == expected.len()
    }

    pub(crate) fn view(&self) -> WorthQueryProvisionalOverlayEvidenceView<'_> {
        WorthQueryProvisionalOverlayEvidenceView { evidence: self }
    }
}

fn expected_proposed_fact(
    action: &super::WorthQueryProvisionalEffectAction,
) -> (&str, WorthQueryProposedFactOrigin) {
    match action {
        super::WorthQueryProvisionalEffectAction::Create { symbolic_identity } => (
            symbolic_identity,
            WorthQueryProposedFactOrigin::StagedCreation,
        ),
        super::WorthQueryProvisionalEffectAction::Replace { target_identity } => (
            target_identity,
            WorthQueryProposedFactOrigin::StagedReplacement,
        ),
        super::WorthQueryProvisionalEffectAction::Retire { target_identity } => (
            target_identity,
            WorthQueryProposedFactOrigin::StagedRetirement,
        ),
        super::WorthQueryProvisionalEffectAction::DeriveView { view_identity } => (
            view_identity,
            WorthQueryProposedFactOrigin::DerivedProvisionalView,
        ),
    }
}

#[derive(Clone, Copy)]
pub struct WorthQueryProvisionalOverlayEvidenceView<'a> {
    evidence: &'a WorthQueryProvisionalOverlayEvidence,
}

impl<'a> WorthQueryProvisionalOverlayEvidenceView<'a> {
    pub(in crate::domain_computation) const fn cleanup_binding(
        self,
    ) -> &'a WorthQueryProvisionalOverlayCleanupBinding {
        &self.evidence.cleanup
    }

    pub fn identity(self) -> &'a str {
        self.evidence.identity()
    }

    pub fn physical_overlay_identity(self) -> &'a str {
        &self.evidence.physical_overlay_identity
    }

    pub fn generation(self) -> u64 {
        self.evidence.generation
    }

    pub fn token_identity(self) -> &'a str {
        &self.evidence.cleanup.token_identity
    }

    pub fn token_generation(self) -> u64 {
        self.evidence.cleanup.token_generation
    }
}

pub trait WorthQueryProvisionalGraphProvider: Send + Sync + 'static {
    fn stage_provisional_overlay(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryProviderSessionView<'_>,
        program: WorthQueryProvisionalEffectProgramView<'_>,
        admission: WorthQueryProvisionalOverlayAdmission,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure>;

    fn discard_provisional_overlay(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), WorthQueryProvisionalFailure>;
}

fn canonical(value: impl Into<Arc<str>>) -> Result<Arc<str>, WorthQueryProvisionalFailure> {
    let value = value.into();
    if value.trim().is_empty() || value.trim() != value.as_ref() {
        return Err(WorthQueryProvisionalFailure::invalid_program(
            "provider overlay evidence must be non-empty canonical text",
        ));
    }
    Ok(value)
}
