use super::{
    budget::{require_compiled_plan_budget, require_terminal_decision_budget},
    cost::compile_cost,
    currentness::{require_classification_currentness, require_scope_currentness},
    effect_compiler::{compile_conflicts, compile_effects, compile_parallel_admission},
    policy::require_policy_session,
    recoverable::{prepare_non_source, UiRebindPlanningRecoveryStop},
    subsystem_compiler::compile_subsystems,
    UiAuthoredContentRebindSemanticProof, UiChangedRebindSemanticProof,
    UiRebindCandidatePreparationDenial, UiRebindConflictFootprint, UiRebindEffectSet,
    UiRebindExecutionPolicy, UiRebindParallelAdmission, UiRebindPlan, UiRebindPlanBasis,
    UiRebindPlanCost, UiRebindPlanInput, UiRebindPlanTarget, UiRebindPlanningContext,
    UiRebindPlanningDenial, UiRebindSemanticProof, UiRebindSubsystemKind, UiRebindSubsystemPlan,
};
use crate::runtime::rebind::UiResolvedIdentityLifecycle;

pub(crate) struct UiRebindPlanCompiler;

impl UiRebindPlanCompiler {
    pub(crate) fn compile(
        context: UiRebindPlanningContext<'_>,
        lifecycle: UiResolvedIdentityLifecycle,
        policy: UiRebindExecutionPolicy,
    ) -> Result<UiRebindPlan, UiRebindPlanningDenial> {
        let (mut scope, identity_decisions) = lifecycle.into_parts();
        require_scope_currentness(&context, &scope)?;
        require_policy_session(context.session(), policy)?;
        let budget = context.budget();
        require_terminal_decision_budget(&identity_decisions, budget)?;
        let semantic_proof = finish_semantic_proof(context.runtime(), &mut scope)?;
        let basis = UiRebindPlanBasis::new(
            scope.basis().classification().clone(),
            semantic_proof_candidate_generation(&semantic_proof)
                .unwrap_or_else(|| scope.basis().candidate_generation())
                .clone(),
        );
        let binding_targets = binding_targets(&semantic_proof);
        let mut subsystems = compile_subsystems(&scope, &identity_decisions, binding_targets);
        if let UiRebindSemanticProof::ThemeSwitch(theme) = &semantic_proof {
            let surface = subsystems
                .iter_mut()
                .find(|row| row.kind() == UiRebindSubsystemKind::Surface)
                .expect("compiled plans include every subsystem");
            *surface = UiRebindSubsystemPlan::new(
                UiRebindSubsystemKind::Surface,
                vec![UiRebindPlanTarget::Surface(
                    theme.prepared().successor().surface(),
                )],
            );
        }
        require_compiled_plan_budget(&scope, &subsystems, budget)?;
        let effects = compile_effects(&subsystems);
        let conflicts = compile_conflicts(&subsystems, &effects);
        let parallel = compile_parallel_admission(&effects);
        let candidate = semantic_proof_candidate_authority(&semantic_proof)
            .unwrap_or_else(|| context.predecessor());
        let content =
            super::content_plan::compile_content_plan(context.predecessor(), candidate, &scope)?;
        let semantic_proof =
            select_authored_content_proof(semantic_proof, &content, &scope, context.predecessor());
        let cost = compile_cost(&identity_decisions, &subsystems, &effects);
        Ok(UiRebindPlan::new(UiRebindPlanInput {
            basis,
            scope: Some(scope),
            identity_decisions,
            subsystems,
            effects,
            conflicts,
            parallel,
            content,
            policy,
            budget,
            effecting_observation_capacity: context.effecting_observation_capacity(),
            cost,
            semantic_proof,
        }))
    }

    pub(crate) fn compile_non_source_recoverable(
        context: UiRebindPlanningContext<'_>,
        lifecycle: UiResolvedIdentityLifecycle,
        policy: UiRebindExecutionPolicy,
    ) -> Result<UiRebindPlan, UiRebindPlanningRecoveryStop> {
        let prepared = match prepare_non_source(&context, &lifecycle, policy) {
            Ok(prepared) => prepared,
            Err(denial) => return Err(UiRebindPlanningRecoveryStop::new(denial, lifecycle)),
        };
        let (scope, identity_decisions) = lifecycle.into_parts();
        let basis = UiRebindPlanBasis::new(
            scope.basis().classification().clone(),
            scope.basis().candidate_generation().clone(),
        );
        Ok(UiRebindPlan::new(UiRebindPlanInput {
            basis,
            scope: Some(scope),
            identity_decisions,
            subsystems: prepared.subsystems,
            effects: prepared.effects,
            conflicts: prepared.conflicts,
            parallel: prepared.parallel,
            content: prepared.content,
            policy,
            budget: context.budget(),
            effecting_observation_capacity: context.effecting_observation_capacity(),
            cost: prepared.cost,
            semantic_proof: UiRebindSemanticProof::NonSource,
        }))
    }

    pub(crate) fn compile_preservation(
        context: UiRebindPlanningContext<'_>,
        evidence: crate::runtime::observation::UiEvidenceOnlySourceChange,
        policy: UiRebindExecutionPolicy,
    ) -> Result<UiRebindPlan, UiRebindPlanningDenial> {
        let (classification, succession) = evidence.into_parts();
        require_classification_currentness(&context, &classification)?;
        require_policy_session(context.session(), policy)?;
        if !matches!(
            succession,
            crate::runtime::observation::UiAuthoredSourceSuccession::EvidenceOnly { .. }
        ) {
            return Err(UiRebindPlanningDenial::WrongSourceSuccessionPosture);
        }
        let basis = UiRebindPlanBasis::new(
            classification,
            succession
                .successor_authority()
                .generation_identity()
                .clone(),
        );
        let subsystems = UiRebindSubsystemKind::all()
            .into_iter()
            .map(|kind| UiRebindSubsystemPlan::new(kind, Vec::new()))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(UiRebindPlan::new(UiRebindPlanInput {
            basis,
            scope: None,
            identity_decisions: Box::new([]),
            subsystems,
            effects: UiRebindEffectSet::new(Vec::new()),
            conflicts: UiRebindConflictFootprint::new(Vec::new(), Vec::new(), Vec::new()),
            parallel: UiRebindParallelAdmission::new(Vec::new()),
            content: crate::mounting::UiMountedSemanticContentInput::empty(),
            policy,
            budget: context.budget(),
            effecting_observation_capacity: context.effecting_observation_capacity(),
            cost: UiRebindPlanCost::new(0, 0, 0, 0, 0),
            semantic_proof: UiRebindSemanticProof::EvidenceOnly(Box::new(succession)),
        }))
    }
}

fn finish_semantic_proof(
    runtime: &crate::runtime::WorthUiRuntime,
    scope: &mut super::super::UiResolvedAffectedScope,
) -> Result<UiRebindSemanticProof, UiRebindPlanningDenial> {
    if let Some(theme) = scope.take_theme_switch() {
        return Ok(UiRebindSemanticProof::ThemeSwitch(Box::new(theme)));
    }
    let Some(succession) = scope.take_source_succession() else {
        return Ok(UiRebindSemanticProof::NonSource);
    };
    let Some((mut candidate, comparison, replacement)) = succession.into_changed_parts() else {
        return Err(UiRebindPlanningDenial::WrongSourceSuccessionPosture);
    };
    let candidate_graph_changed_nodes =
        candidate.prepare_rebind_mount_eligibility().map_err(|_| {
            UiRebindPlanningDenial::CandidatePreparation(
                UiRebindCandidatePreparationDenial::MountEligibility,
            )
        })?;
    let lowering = runtime
        .finish_precomputed_replacement_lowering(replacement, &candidate)
        .map_err(|denial| UiRebindPlanningDenial::Replacement(Box::new(denial)))?;
    Ok(UiRebindSemanticProof::Changed(Box::new(
        UiChangedRebindSemanticProof {
            successor_authority: candidate,
            lowering,
            candidate_graph_changed_nodes,
            artifact_comparison: comparison.outcome(),
        },
    )))
}

fn select_authored_content_proof(
    proof: UiRebindSemanticProof,
    content: &crate::mounting::UiMountedSemanticContentInput,
    scope: &super::super::UiResolvedAffectedScope,
    predecessor: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
) -> UiRebindSemanticProof {
    let UiRebindSemanticProof::Changed(changed) = proof else {
        return proof;
    };
    let is_equivalent = changed.artifact_comparison
        == crate::runtime::WorthUiRuntimeArtifactComparisonOutcome::EquivalentNoOp;
    let transitions = content.schema_transitions();
    let is_schema_recovery = !transitions.is_empty()
        && transitions.iter().all(|transition| {
            transition.kind() == super::UiProjectionSchemaTransitionKind::Recovered
        });
    let only_content_changed = scope
        .affected_aspects()
        .iter()
        .all(|aspect| aspect.family() == crate::declaration::UiAspectFamily::Content);
    let attachments = |authority: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority| {
        authority.graph_snapshot().nodes().iter().map(|node| (
            (node.declaration_identity().authored_semantic_name().to_owned(), node.repeated_instance_basis().identity_digest()),
            node.appearance_role_attachment().cloned(),
        )).collect::<std::collections::BTreeMap<_, _>>()
    };
    let same_attachments = attachments(predecessor) == attachments(&changed.successor_authority);
    if !is_equivalent || !is_schema_recovery || !only_content_changed || !same_attachments {
        return UiRebindSemanticProof::Changed(changed);
    }
    let source_candidate_artifact_digest = changed
        .lowering
        .admitted()
        .candidate()
        .basis()
        .artifact_digest()
        .raw();
    UiRebindSemanticProof::AuthoredContent(Box::new(UiAuthoredContentRebindSemanticProof {
        successor_authority: changed.successor_authority,
        source_candidate_artifact_digest,
    }))
}

fn binding_targets(proof: &UiRebindSemanticProof) -> Vec<UiRebindPlanTarget> {
    let UiRebindSemanticProof::Changed(changed) = proof else {
        return Vec::new();
    };
    changed
        .lowering
        .query_rebind_plan()
        .entries()
        .iter()
        .map(|entry| UiRebindPlanTarget::QueryBinding(entry.identity().view_binding_id().into()))
        .collect()
}

fn semantic_proof_candidate_generation(
    proof: &UiRebindSemanticProof,
) -> Option<
    &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
> {
    match proof {
        UiRebindSemanticProof::Changed(changed) => {
            Some(changed.successor_authority.generation_identity())
        }
        UiRebindSemanticProof::AuthoredContent(content) => {
            Some(content.successor_authority.generation_identity())
        }
        UiRebindSemanticProof::EvidenceOnly(succession) => {
            Some(succession.successor_authority().generation_identity())
        }
        UiRebindSemanticProof::ThemeSwitch(_) | UiRebindSemanticProof::NonSource => None,
        UiRebindSemanticProof::Transferred => {
            unreachable!("planning never receives a transferred semantic proof")
        }
    }
}

fn semantic_proof_candidate_authority(
    proof: &UiRebindSemanticProof,
) -> Option<&crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority> {
    match proof {
        UiRebindSemanticProof::Changed(changed) => Some(&changed.successor_authority),
        UiRebindSemanticProof::AuthoredContent(content) => Some(&content.successor_authority),
        UiRebindSemanticProof::EvidenceOnly(succession) => Some(succession.successor_authority()),
        UiRebindSemanticProof::ThemeSwitch(_)
        | UiRebindSemanticProof::NonSource
        | UiRebindSemanticProof::Transferred => None,
    }
}

impl UiRebindSubsystemKind {
    pub(super) const fn all() -> [Self; 9] {
        [
            Self::Preservation,
            Self::Graph,
            Self::Mount,
            Self::Measurement,
            Self::Allocation,
            Self::Binding,
            Self::Obligation,
            Self::Surface,
            Self::Retirement,
        ]
    }
}
