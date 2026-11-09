use super::*;

pub(in crate::harness::milestone_eight_certification) fn support_profile_bundle(
    enabled: bool,
) -> MilestoneEightCertificationBundle {
    let facade = if enabled {
        WorthQueryApplicationFacade::runtime_backed_default()
    } else {
        WorthQueryApplicationFacade::new(
            crate::application::WorthQueryConfig::runtime_backed_default()
                .with_query(crate::application::WorthQueryQueryConfig::disabled())
                .with_signal(crate::application::WorthQuerySignalConfig::disabled())
                .with_runtime_bridge(crate::application::WorthQueryRuntimeBridgeConfig::disabled())
                .with_relational(crate::application::WorthQueryRelationalConfig::disabled()),
        )
        .expect("disabled query config should still admit a support-report facade")
    };
    let report = facade.support_report().expect("fixture digest must fit");
    let composition_profile = report
        .query_composition_support_profile()
        .map(|profile| profile.profile_digest().to_string())
        .unwrap_or_else(|| "none".to_string());
    let identity_evolution_profile = report
        .identity_evolution_support_profile()
        .map(|profile| profile.profile_digest().to_string())
        .unwrap_or_else(|| "none".to_string());
    let query_context_profile = report
        .query_context_support_profile()
        .map(|profile| profile.profile_digest().to_string())
        .unwrap_or_else(|| "none".to_string());
    bundle_from_view_execution(
        report.report_digest().to_string(),
        report.support_matrix().support_matrix_digest().to_string(),
        report.validated_config_digest().to_string(),
        report.report_digest().to_string(),
        vec![
            format!("admitted:{}", report.admitted_capability_count()),
            format!("deferred:{}", report.deferred_capability_count()),
            format!("unsupported:{}", report.unsupported_capability_count()),
            format!("query_composition_profile:{composition_profile}"),
            format!("query_context_profile:{query_context_profile}"),
            format!("identity_evolution_profile:{identity_evolution_profile}"),
        ],
        digest_parts(&[
            report.support_matrix().support_matrix_digest().to_string(),
            composition_profile,
            query_context_profile,
            identity_evolution_profile,
        ]),
        report.report_digest().to_string(),
    )
}
