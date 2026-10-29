pub(crate) use worth_query_worlds::facade::publication::application_authorization_publication::profile_progression::*;

#[test]
fn public_denial_publisher_retains_explicit_two_stage_profile_narrowing() {
    let denial = real_denial(CompositionScenario::MissingAuthorization);
    let requested = profile(DiagnosticRichnessProfile::Forensic);
    let admitted = profile(DiagnosticRichnessProfile::Standard);
    let materialized = profile(DiagnosticRichnessProfile::OperationalMinimal);
    let publication_profile =
        WorthQueryApplicationAuthorizationPublicationProfile::with_progression(
            requested,
            stage(admitted, "admission retained standard diagnostics"),
            stage(materialized, "delivery retained operational diagnostics"),
        );

    let published = publish_application_authorization_denial(&denial, publication_profile).unwrap();
    let progression = published.boundary().payload().profile();
    assert_eq!(progression.requested(), &requested);
    assert_eq!(progression.admitted(), &admitted);
    assert_eq!(progression.materialized(), &materialized);
    assert!(progression.requested_to_admitted_narrowing().is_some());
    assert!(progression.admitted_to_materialized_narrowing().is_some());
}

#[test]
fn public_denial_publisher_rejects_admission_and_materialization_widening_separately() {
    let denial = real_denial(CompositionScenario::MissingAuthorization);
    let standard = profile(DiagnosticRichnessProfile::Standard);
    let forensic = profile(DiagnosticRichnessProfile::Forensic);
    let claimed_narrowing = || {
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "the hostile profile falsely claims narrowing",
        ))
    };

    let admission = WorthQueryApplicationAuthorizationPublicationProfile::with_progression(
        standard,
        WorthQueryApplicationAuthorizationProfileStage::new(forensic, claimed_narrowing()),
        WorthQueryApplicationAuthorizationProfileStage::new(forensic, None),
    );
    assert_eq!(
        publish_application_authorization_denial(&denial, admission).unwrap_err(),
        WorthQueryApplicationAuthorizationPublicationDenial::ProfileAdmission(
            FoundationalProfileProgressionDenial::RequestedAndAdmittedProfilesMayOnlyNarrow,
        )
    );

    let materialization = WorthQueryApplicationAuthorizationPublicationProfile::with_progression(
        standard,
        WorthQueryApplicationAuthorizationProfileStage::new(standard, None),
        WorthQueryApplicationAuthorizationProfileStage::new(forensic, claimed_narrowing()),
    );
    assert_eq!(
        publish_application_authorization_denial(&denial, materialization).unwrap_err(),
        WorthQueryApplicationAuthorizationPublicationDenial::ProfileMaterialization(
            FoundationalProfileProgressionDenial::RequestedAndAdmittedProfilesMayOnlyNarrow,
        )
    );
}
