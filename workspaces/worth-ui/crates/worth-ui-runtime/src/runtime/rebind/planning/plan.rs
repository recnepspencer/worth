use super::{
    UiRebindConflictFootprint, UiRebindEffectSet, UiRebindExecutionPolicy,
    UiRebindParallelAdmission, UiRebindPlanBasis, UiRebindPlanCost, UiRebindSubsystemKind,
    UiRebindSubsystemPlan,
};

pub struct UiRebindPlan {
    basis: UiRebindPlanBasis,
    scope: Option<super::super::UiResolvedAffectedScope>,
    identity_decisions: Box<[super::super::UiIdentityLifecycleEntry]>,
    subsystems: Box<[UiRebindSubsystemPlan]>,
    semantic_surface_targets: Box<[super::UiRebindPlanTarget]>,
    effects: UiRebindEffectSet,
    conflicts: UiRebindConflictFootprint,
    parallel: UiRebindParallelAdmission,
    content: crate::mounting::UiMountedSemanticContentInput,
    policy: UiRebindExecutionPolicy,
    budget: crate::runtime::rebind::UiRebindBudgetInput,
    effecting_observation_capacity: usize,
    cost: UiRebindPlanCost,
    source_candidate_artifact_digest: Option<u64>,
    semantic_proof: UiRebindSemanticProof,
}

pub(crate) enum UiRebindSemanticProof {
    Changed(Box<UiChangedRebindSemanticProof>),
    AuthoredContent(Box<UiAuthoredContentRebindSemanticProof>),
    EvidenceOnly(Box<crate::runtime::observation::UiAuthoredSourceSuccession>),
    ThemeSwitch(Box<crate::runtime::appearance::UiThemeSwitchChange>),
    NonSource,
    Transferred,
}

pub(crate) struct UiChangedRebindSemanticProof {
    pub(crate) successor_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    pub(crate) lowering: crate::runtime::WorthUiReplacementLoweringReady,
    pub(crate) candidate_graph_changed_nodes:
        std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
    pub(crate) artifact_comparison: crate::runtime::WorthUiRuntimeArtifactComparisonOutcome,
}

pub(crate) struct UiAuthoredContentRebindSemanticProof {
    pub(crate) successor_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    pub(crate) source_candidate_artifact_digest: u64,
}

pub(crate) struct UiRebindPlanInput {
    pub(crate) basis: UiRebindPlanBasis,
    pub(crate) scope: Option<super::super::UiResolvedAffectedScope>,
    pub(crate) identity_decisions: Box<[super::super::UiIdentityLifecycleEntry]>,
    pub(crate) subsystems: Box<[UiRebindSubsystemPlan]>,
    pub(crate) effects: UiRebindEffectSet,
    pub(crate) conflicts: UiRebindConflictFootprint,
    pub(crate) parallel: UiRebindParallelAdmission,
    pub(crate) content: crate::mounting::UiMountedSemanticContentInput,
    pub(crate) policy: UiRebindExecutionPolicy,
    pub(crate) budget: crate::runtime::rebind::UiRebindBudgetInput,
    pub(crate) effecting_observation_capacity: usize,
    pub(crate) cost: UiRebindPlanCost,
    pub(crate) semantic_proof: UiRebindSemanticProof,
}

impl UiRebindPlan {
    pub(crate) fn new(input: UiRebindPlanInput) -> Self {
        let source_candidate_artifact_digest =
            semantic_proof_source_candidate_artifact_digest(&input.semantic_proof);
        Self {
            basis: input.basis,
            scope: input.scope,
            identity_decisions: input.identity_decisions,
            semantic_surface_targets: input
                .subsystems
                .iter()
                .find(|subsystem| subsystem.kind() == UiRebindSubsystemKind::Surface)
                .map(|subsystem| subsystem.targets().to_vec().into_boxed_slice())
                .unwrap_or_default(),
            subsystems: input.subsystems,
            effects: input.effects,
            conflicts: input.conflicts,
            parallel: input.parallel,
            content: input.content,
            policy: input.policy,
            budget: input.budget,
            effecting_observation_capacity: input.effecting_observation_capacity,
            cost: input.cost,
            source_candidate_artifact_digest,
            semantic_proof: input.semantic_proof,
        }
    }

    pub const fn basis(&self) -> &UiRebindPlanBasis {
        &self.basis
    }

    pub fn scope(&self) -> Option<&super::super::UiResolvedAffectedScope> {
        self.scope.as_ref()
    }

    pub(crate) fn scalar_projection_fact_count(&self) -> usize {
        self.scope
            .as_ref()
            .into_iter()
            .flat_map(|scope| scope.facts())
            .filter(|fact| {
                fact.query()
                    .and_then(crate::fact_contract::UiQueryChangedFact::scalar_projection)
                    .is_some()
            })
            .count()
    }

    pub(crate) fn into_scalar_projection_fact(
        self,
    ) -> Option<worth_ui_query_binding::UiScalarProjectionFactReceipt> {
        self.scope?
            .into_facts()
            .into_vec()
            .into_iter()
            .find_map(|fact| fact.into_scalar_projection().ok())
    }

    pub(crate) fn application_scalar_projection_fact_count(&self) -> usize {
        self.scope
            .as_ref()
            .into_iter()
            .flat_map(|scope| scope.facts())
            .filter(|fact| {
                fact.query()
                    .and_then(
                        crate::fact_contract::UiQueryChangedFact::application_scalar_projection,
                    )
                    .is_some()
            })
            .count()
    }

    pub(crate) fn into_application_scalar_projection_fact(
        self,
    ) -> Option<worth_ui_query_binding::UiApplicationScalarProjectionFactReceipt> {
        self.scope?
            .into_facts()
            .into_vec()
            .into_iter()
            .find_map(|fact| fact.into_application_scalar_projection().ok())
    }

    pub fn identity_decisions(&self) -> &[super::super::UiIdentityLifecycleEntry] {
        &self.identity_decisions
    }

    pub fn subsystems(&self) -> &[UiRebindSubsystemPlan] {
        &self.subsystems
    }

    pub fn subsystem(&self, kind: UiRebindSubsystemKind) -> Option<&UiRebindSubsystemPlan> {
        self.subsystems
            .binary_search_by_key(&kind, UiRebindSubsystemPlan::kind)
            .ok()
            .map(|index| &self.subsystems[index])
    }

    pub const fn effects(&self) -> &UiRebindEffectSet {
        &self.effects
    }

    pub const fn conflicts(&self) -> &UiRebindConflictFootprint {
        &self.conflicts
    }

    pub const fn parallel_admission(&self) -> &UiRebindParallelAdmission {
        &self.parallel
    }

    pub(crate) const fn content(&self) -> &crate::mounting::UiMountedSemanticContentInput {
        &self.content
    }

    pub fn projection_schema_transitions(&self) -> &[super::UiProjectionSchemaTransition] {
        self.content.schema_transitions()
    }

    pub const fn execution_policy(&self) -> UiRebindExecutionPolicy {
        self.policy
    }

    pub const fn budget(&self) -> crate::runtime::rebind::UiRebindBudgetInput {
        self.budget
    }

    pub const fn effecting_observation_capacity(&self) -> usize {
        self.effecting_observation_capacity
    }

    pub const fn cost(&self) -> UiRebindPlanCost {
        self.cost
    }

    pub fn source_candidate_artifact_digest(&self) -> Option<u64> {
        self.source_candidate_artifact_digest
    }

    pub(crate) fn semantic_candidate_generation(
        &self,
    ) -> Option<
        &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
    >{
        match &self.semantic_proof {
            UiRebindSemanticProof::Changed(changed) => {
                Some(changed.successor_authority.generation_identity())
            }
            UiRebindSemanticProof::AuthoredContent(content) => {
                Some(content.successor_authority.generation_identity())
            }
            UiRebindSemanticProof::EvidenceOnly(succession) => {
                Some(succession.successor_authority().generation_identity())
            }
            UiRebindSemanticProof::ThemeSwitch(_)
            | UiRebindSemanticProof::NonSource
            | UiRebindSemanticProof::Transferred => None,
        }
    }

    pub(crate) fn take_semantic_proof(&mut self) -> UiRebindSemanticProof {
        std::mem::replace(&mut self.semantic_proof, UiRebindSemanticProof::Transferred)
    }

    pub(crate) fn retained_successor_authority(
        &self,
    ) -> Option<&crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority>
    {
        match &self.semantic_proof {
            UiRebindSemanticProof::AuthoredContent(content) => Some(&content.successor_authority),
            UiRebindSemanticProof::EvidenceOnly(succession) => {
                Some(succession.successor_authority())
            }
            _ => None,
        }
    }

    pub(crate) fn include_surface_consumers(&mut self, targets: Vec<super::UiRebindPlanTarget>) {
        let surface = self
            .subsystems
            .iter_mut()
            .find(|subsystem| subsystem.kind() == UiRebindSubsystemKind::Surface)
            .expect("every compiled plan includes its surface subsystem");
        *surface = UiRebindSubsystemPlan::new(
            UiRebindSubsystemKind::Surface,
            self.semantic_surface_targets
                .iter()
                .cloned()
                .chain(targets)
                .collect(),
        );
        self.effects = super::effect_compiler::compile_effects(&self.subsystems);
        self.conflicts = super::effect_compiler::compile_conflicts(&self.subsystems, &self.effects);
        self.parallel = super::effect_compiler::compile_parallel_admission(&self.effects);
        self.cost =
            super::cost::compile_cost(&self.identity_decisions, &self.subsystems, &self.effects);
    }

    pub(crate) const fn has_non_source_semantic_proof(&self) -> bool {
        matches!(self.semantic_proof, UiRebindSemanticProof::NonSource)
    }

    pub(crate) fn into_retained_facts(self) -> Box<[crate::fact_contract::UiProducedFact]> {
        self.scope.map_or_else(
            || Box::new([]),
            crate::runtime::rebind::UiResolvedAffectedScope::into_facts,
        )
    }

    #[cfg(test)]
    pub(crate) const fn semantic_proof(&self) -> &UiRebindSemanticProof {
        &self.semantic_proof
    }
}

fn semantic_proof_source_candidate_artifact_digest(proof: &UiRebindSemanticProof) -> Option<u64> {
    match proof {
        UiRebindSemanticProof::Changed(changed) => Some(
            changed
                .lowering
                .admitted()
                .candidate()
                .basis()
                .artifact_digest()
                .raw(),
        ),
        UiRebindSemanticProof::AuthoredContent(content) => {
            Some(content.source_candidate_artifact_digest)
        }
        UiRebindSemanticProof::EvidenceOnly(succession) => succession
            .admitted_candidate()
            .map(|admitted| admitted.candidate().basis().artifact_digest().raw()),
        UiRebindSemanticProof::ThemeSwitch(_)
        | UiRebindSemanticProof::NonSource
        | UiRebindSemanticProof::Transferred => None,
    }
}
