use {worth_foundational::materialize_diagnostic_explanation_bundle, worth_foundational::materialize_diagnostic_support_report, worth_foundational::plan_diagnostic_explanation_bundle, worth_foundational::plan_diagnostic_support_report, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalDiagnosticAssemblyDebt, worth_foundational::FoundationalDiagnosticAssemblyDebtClass, worth_foundational::FoundationalDiagnosticDeliveryClass, worth_foundational::FoundationalDiagnosticExplanationInput, worth_foundational::FoundationalDiagnosticGapClass, worth_foundational::FoundationalDiagnosticGapClosurePosture, worth_foundational::FoundationalDiagnosticGapTarget, worth_foundational::FoundationalDiagnosticMaterializationDenial, worth_foundational::FoundationalDiagnosticNamedGap, worth_foundational::FoundationalDiagnosticOutcomeKind, worth_foundational::FoundationalDiagnosticPartiality, worth_foundational::FoundationalDiagnosticSupportClaimStrength, worth_foundational::FoundationalDiagnosticSupportInput, worth_foundational::FoundationalDiagnosticSurfaceAvailability, worth_foundational::SupportPostureProfile};

use {crate::certification::ordinary_context::diagnostics::materialization_support::counters, crate::certification::ordinary_context::diagnostics::materialization_support::explanation_input, crate::certification::ordinary_context::diagnostics::materialization_support::locator, crate::certification::ordinary_context::diagnostics::materialization_support::profile, crate::certification::ordinary_context::diagnostics::materialization_support::required_row, crate::certification::ordinary_context::diagnostics::materialization_support::row_codes, crate::certification::ordinary_context::diagnostics::materialization_support::standard_row, crate::certification::ordinary_context::diagnostics::materialization_support::subject, crate::certification::ordinary_context::diagnostics::materialization_support::support_input};

#[test]
fn reduced_richness_changes_breadth_without_changing_outcome_truth() {
    let operational = materialize_diagnostic_explanation_bundle(
        explanation_input(worth_foundational::FoundationalDiagnosticPartiality::Complete),
        profile(
            worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("operational explanation materializes");
    let forensic = materialize_diagnostic_explanation_bundle(
        explanation_input(worth_foundational::FoundationalDiagnosticPartiality::Complete),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Forensic,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("forensic explanation materializes");

    assert_eq!(operational.outcome_kind(), forensic.outcome_kind());
    assert_eq!(row_codes(operational.rows()), vec!["support.required"]);
    assert_eq!(
        row_codes(forensic.rows()),
        vec!["support.forensic", "support.required", "support.standard"]
    );
}

#[test]
fn support_reports_reject_overclaiming_durable_or_certified_support() {
    assert_eq!(
        plan_diagnostic_support_report(
            support_input(
                worth_foundational::FoundationalDiagnosticSupportClaimStrength::DurableSupportReady,
                worth_foundational::FoundationalDiagnosticPartiality::Complete,
                vec![],
            ),
            profile(
                worth_foundational::DiagnosticRichnessProfile::Standard,
                worth_foundational::SupportPostureProfile::InternalOnly,
            ),
            worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(worth_foundational::FoundationalDiagnosticMaterializationDenial::InternalSupportCannotClaimDurableSupport)
    );

    assert_eq!(
        plan_diagnostic_support_report(
            support_input(
                worth_foundational::FoundationalDiagnosticSupportClaimStrength::CertifiedSupportReady,
                worth_foundational::FoundationalDiagnosticPartiality::Complete,
                vec![],
            ),
            profile(
                worth_foundational::DiagnosticRichnessProfile::Standard,
                worth_foundational::SupportPostureProfile::CertificationReady,
            ),
            worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(
            worth_foundational::FoundationalDiagnosticMaterializationDenial::CertifiedSupportRequiresProductionCertifiedProfile
        )
    );

    assert_eq!(
        plan_diagnostic_support_report(
            worth_foundational::FoundationalDiagnosticSupportInput::new(
                subject(),
                worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
                vec![],
                vec![standard_row()],
                vec![],
                worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
                worth_foundational::FoundationalDiagnosticSupportClaimStrength::DurableSupportReady,
                worth_foundational::FoundationalDiagnosticPartiality::Complete,
                counters(),
                vec![],
            ),
            profile(
                worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
                worth_foundational::SupportPostureProfile::SupportReady,
            ),
            worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(
            worth_foundational::FoundationalDiagnosticMaterializationDenial::DurableSupportRequiresVisibleRowsAtChosenRichness
        )
    );
}

#[test]
fn reconstruction_and_unavailability_stay_explicit_in_materialized_surfaces() {
    let reconstructable = materialize_diagnostic_explanation_bundle(
        worth_foundational::FoundationalDiagnosticExplanationInput::new(
            subject(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Deferred,
            vec![required_row()],
            vec![],
            vec![],
            worth_foundational::FoundationalDiagnosticSurfaceAvailability::reconstructable(),
            worth_foundational::FoundationalDiagnosticPartiality::Complete,
            counters(),
            vec![],
        ),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
    )
    .expect("reconstructable explanation materializes");
    let unavailable = materialize_diagnostic_explanation_bundle(
        worth_foundational::FoundationalDiagnosticExplanationInput::new(
            subject(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Unsupported,
            vec![required_row()],
            vec![],
            vec![],
            worth_foundational::FoundationalDiagnosticSurfaceAvailability::unavailable(
                worth_foundational::FoundationalDiagnosticAbsenceCause::MissingEvidence,
            ),
            worth_foundational::FoundationalDiagnosticPartiality::Complete,
            counters(),
            vec![],
        ),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::UnavailableByPolicy,
    )
    .expect("unavailable explanation materializes");

    assert_eq!(
        reconstructable.availability().availability(),
        worth_foundational::FoundationalDiagnosticAvailability::Reconstructable
    );
    assert_eq!(
        unavailable.availability().absence_cause(),
        Some(worth_foundational::FoundationalDiagnosticAbsenceCause::MissingEvidence)
    );
}

#[test]
fn fallback_debt_and_repeated_rediscovery_remain_explicit() {
    let plan = plan_diagnostic_explanation_bundle(
        explanation_input(worth_foundational::FoundationalDiagnosticPartiality::Complete),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation plan");

    assert_eq!(plan.assembly_debts().len(), 2);
    assert_eq!(
        plan.assembly_debts()[0].class(),
        worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback
    );
    assert_eq!(plan.assembly_debts()[0].count(), 2);
    assert_eq!(
        plan.assembly_debts()[1].class(),
        worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RepeatedRediscovery
    );
    assert_eq!(plan.assembly_debts()[1].count(), 1);
}

#[test]
fn partial_support_uses_typed_named_gaps() {
    let gap = worth_foundational::FoundationalDiagnosticNamedGap::new(
        worth_foundational::FoundationalDiagnosticGapClass::CoverageOmission,
        worth_foundational::FoundationalDiagnosticGapTarget::Locator(locator()),
        worth_foundational::FoundationalDiagnosticGapClosurePosture::DebtNamed,
    );
    let report = materialize_diagnostic_support_report(
        support_input(
            worth_foundational::FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            worth_foundational::FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![gap.clone()]),
            vec![],
        ),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("partial support report");

    assert_eq!(report.named_gaps(), &[gap]);
}

#[test]
fn named_gap_shape_rejects_empty_partiality() {
    assert_eq!(
        plan_diagnostic_explanation_bundle(
            explanation_input(worth_foundational::FoundationalDiagnosticPartiality::PartialWithNamedGaps(
                vec![]
            )),
            profile(
                worth_foundational::DiagnosticRichnessProfile::Standard,
                worth_foundational::SupportPostureProfile::SupportReady
            ),
            worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(worth_foundational::FoundationalDiagnosticMaterializationDenial::PartialityRequiresNamedGaps)
    );
}

#[test]
fn zero_count_fallback_debt_is_rejected_as_fake_explicit_cost() {
    assert_eq!(
        plan_diagnostic_explanation_bundle(
            worth_foundational::FoundationalDiagnosticExplanationInput::new(
                subject(),
                worth_foundational::FoundationalDiagnosticOutcomeKind::Deferred,
                vec![required_row()],
                vec![],
                vec![],
                worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
                worth_foundational::FoundationalDiagnosticPartiality::Complete,
                counters(),
                vec![worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                    worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                    0,
                )],
            ),
            profile(
                worth_foundational::DiagnosticRichnessProfile::Standard,
                worth_foundational::SupportPostureProfile::SupportReady,
            ),
            worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(worth_foundational::FoundationalDiagnosticMaterializationDenial::RowScanFallbackMustRemainExplicitDebt)
    );
}
