use super::*;

#[test]
fn identity_evolution_capability_disables_typed_and_early_when_query_section_is_off() {
    let facade = WorthQueryApplicationFacade::new(
        WorthQueryConfig::runtime_backed_default()
            .with_query(WorthQueryQueryConfig::disabled())
            .with_signal(WorthQuerySignalConfig::disabled())
            .with_runtime_bridge(WorthQueryRuntimeBridgeConfig::disabled())
            .with_relational(
                WorthQueryRelationalConfig::disabled().with_historical_evaluation(false),
            ),
    )
    .expect("query-disabled facade config should still validate");

    let error = facade
        .identity_evolution_capability()
        .expect_err("disabled query section should deny identity evolution");
    assert_eq!(
        error.failure_class(),
        WorthQueryFacadeFailureClass::MissingOwningSection
    );
    assert_eq!(
        error.capability_family(),
        Some(WorthQueryCapabilityFamily::IdentityEvolution)
    );
}

#[test]
fn support_report_tracks_disabled_signal_section_posture() {
    let facade = WorthQueryApplicationFacade::new(
        WorthQueryConfig::runtime_backed_default().with_signal(WorthQuerySignalConfig::disabled()),
    )
    .expect("signal-disabled facade config should still validate");
    let report = facade.support_report().expect("fixture digest must fit");

    let signal_posture = report
        .section_postures()
        .iter()
        .find(|posture| posture.section() == WorthQueryConfigSectionFamily::Signal)
        .expect("signal posture should be present");
    assert_eq!(
        signal_posture.owner(),
        super::super::WorthQuerySubsystemOwner::Signal
    );
    assert!(!signal_posture.enabled());
    assert!(report
        .unsupported_capability_families()
        .contains(&WorthQueryCapabilityFamily::LiveQuery));
}
