use super::*;

pub(super) fn seal_prepared_activation(
    evidence: WorthUiPreparedCutoverEvidence,
    activation: crate::runtime::WorthUiPreparedApplicationPlanSwap,
) -> WorthUiPreparedApplicationCutoverOutcome {
    let successor_runtime = activation.candidate_runtime_observation();
    let publication = WorthUiApplicationPublicationObservation::prepare_successor(
        WorthUiApplicationPublicationPreparation {
            application_generation: evidence.generations.active.clone(),
            successor_runtime: successor_runtime.clone(),
            runtime_basis: evidence.runtime_basis,
            host_session: evidence.host_session,
            successor_scheduler: activation.candidate_scheduler_state(),
        },
    );
    let reload_cost = evidence.reload_cost_seed.finish(
        evidence.generations.prior.clone(),
        evidence.generations.active.clone(),
        activation.previous_active_plan_digest(),
        successor_runtime
            .cross_lane_bundle()
            .construction_counters(),
        activation
            .plan_decision()
            .summary()
            .expect("prepared activation carries comparison evidence"),
    );
    WorthUiPreparedApplicationCutoverOutcome::Activation(Box::new(
        WorthUiPreparedApplicationActivation {
            identity: Box::new(WorthUiApplicationCutoverIdentityEvidence {
                prior_generation: evidence.generations.prior,
                active_generation: evidence.generations.active,
            }),
            publication: Box::new(publication),
            visual_trace_source: evidence.visual_trace_source,
            font_collection: evidence.font_collection,
            candidate_graph: evidence.candidate_graph,
            candidate_application_authority: evidence.candidate_application_authority,
            candidate_service_policy_plan: evidence.candidate_service_policy_plan,
            appearance_theme_succession: Some(evidence.appearance_theme),
            reload_cost,
            transition: Some(WorthUiApplicationCutoverTransition::Prepared(activation)),
        },
    ))
}
