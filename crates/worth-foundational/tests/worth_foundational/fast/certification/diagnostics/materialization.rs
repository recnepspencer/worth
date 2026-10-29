use worth_foundational::{
    materialize_diagnostic_explanation_bundle, materialize_diagnostic_support_report,
    plan_diagnostic_explanation_bundle, plan_diagnostic_support_report, DiagnosticRichnessProfile,
    FoundationalDiagnosticAssemblyDebt, FoundationalDiagnosticAssemblyDebtClass,
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticExplanationInput,
    FoundationalDiagnosticGapClass, FoundationalDiagnosticGapClosurePosture,
    FoundationalDiagnosticGapTarget, FoundationalDiagnosticMaterializationDenial,
    FoundationalDiagnosticNamedGap, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticPartiality, FoundationalDiagnosticSupportClaimStrength,
    FoundationalDiagnosticSupportInput, FoundationalDiagnosticSurfaceAvailability,
    SupportPostureProfile,
};

use super::materialization_support::{
    counters, explanation_input, locator, profile, required_row, row_codes, standard_row, subject,
    support_input,
};

#[test]
fn reduced_richness_changes_breadth_without_changing_outcome_truth() {
    let operational = materialize_diagnostic_explanation_bundle(
        explanation_input(FoundationalDiagnosticPartiality::Complete),
        profile(
            DiagnosticRichnessProfile::OperationalMinimal,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("operational explanation materializes");
    let forensic = materialize_diagnostic_explanation_bundle(
        explanation_input(FoundationalDiagnosticPartiality::Complete),
        profile(
            DiagnosticRichnessProfile::Forensic,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
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
                FoundationalDiagnosticSupportClaimStrength::DurableSupportReady,
                FoundationalDiagnosticPartiality::Complete,
                vec![],
            ),
            profile(
                DiagnosticRichnessProfile::Standard,
                SupportPostureProfile::InternalOnly,
            ),
            FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(FoundationalDiagnosticMaterializationDenial::InternalSupportCannotClaimDurableSupport)
    );

    assert_eq!(
        plan_diagnostic_support_report(
            support_input(
                FoundationalDiagnosticSupportClaimStrength::CertifiedSupportReady,
                FoundationalDiagnosticPartiality::Complete,
                vec![],
            ),
            profile(
                DiagnosticRichnessProfile::Standard,
                SupportPostureProfile::CertificationReady,
            ),
            FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(
            FoundationalDiagnosticMaterializationDenial::CertifiedSupportRequiresProductionCertifiedProfile
        )
    );

    assert_eq!(
        plan_diagnostic_support_report(
            FoundationalDiagnosticSupportInput::new(
                subject(),
                FoundationalDiagnosticOutcomeKind::Accepted,
                vec![],
                vec![standard_row()],
                vec![],
                FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
                FoundationalDiagnosticSupportClaimStrength::DurableSupportReady,
                FoundationalDiagnosticPartiality::Complete,
                counters(),
                vec![],
            ),
            profile(
                DiagnosticRichnessProfile::OperationalMinimal,
                SupportPostureProfile::SupportReady,
            ),
            FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(
            FoundationalDiagnosticMaterializationDenial::DurableSupportRequiresVisibleRowsAtChosenRichness
        )
    );
}

#[test]
fn reconstruction_and_unavailability_stay_explicit_in_materialized_surfaces() {
    let reconstructable = materialize_diagnostic_explanation_bundle(
        FoundationalDiagnosticExplanationInput::new(
            subject(),
            FoundationalDiagnosticOutcomeKind::Deferred,
            vec![required_row()],
            vec![],
            vec![],
            FoundationalDiagnosticSurfaceAvailability::reconstructable(),
            FoundationalDiagnosticPartiality::Complete,
            counters(),
            vec![],
        ),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
    )
    .expect("reconstructable explanation materializes");
    let unavailable = materialize_diagnostic_explanation_bundle(
        FoundationalDiagnosticExplanationInput::new(
            subject(),
            FoundationalDiagnosticOutcomeKind::Unsupported,
            vec![required_row()],
            vec![],
            vec![],
            FoundationalDiagnosticSurfaceAvailability::unavailable(
                worth_foundational::FoundationalDiagnosticAbsenceCause::MissingEvidence,
            ),
            FoundationalDiagnosticPartiality::Complete,
            counters(),
            vec![],
        ),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::UnavailableByPolicy,
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
        explanation_input(FoundationalDiagnosticPartiality::Complete),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation plan");

    assert_eq!(plan.assembly_debts().len(), 2);
    assert_eq!(
        plan.assembly_debts()[0].class(),
        FoundationalDiagnosticAssemblyDebtClass::RowScanFallback
    );
    assert_eq!(plan.assembly_debts()[0].count(), 2);
    assert_eq!(
        plan.assembly_debts()[1].class(),
        FoundationalDiagnosticAssemblyDebtClass::RepeatedRediscovery
    );
    assert_eq!(plan.assembly_debts()[1].count(), 1);
}

#[test]
fn partial_support_uses_typed_named_gaps() {
    let gap = FoundationalDiagnosticNamedGap::new(
        FoundationalDiagnosticGapClass::CoverageOmission,
        FoundationalDiagnosticGapTarget::Locator(locator()),
        FoundationalDiagnosticGapClosurePosture::DebtNamed,
    );
    let report = materialize_diagnostic_support_report(
        support_input(
            FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![gap.clone()]),
            vec![],
        ),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("partial support report");

    assert_eq!(report.named_gaps(), &[gap]);
}

#[test]
fn named_gap_shape_rejects_empty_partiality() {
    assert_eq!(
        plan_diagnostic_explanation_bundle(
            explanation_input(FoundationalDiagnosticPartiality::PartialWithNamedGaps(
                vec![]
            )),
            profile(
                DiagnosticRichnessProfile::Standard,
                SupportPostureProfile::SupportReady
            ),
            FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(FoundationalDiagnosticMaterializationDenial::PartialityRequiresNamedGaps)
    );
}

#[test]
fn zero_count_fallback_debt_is_rejected_as_fake_explicit_cost() {
    assert_eq!(
        plan_diagnostic_explanation_bundle(
            FoundationalDiagnosticExplanationInput::new(
                subject(),
                FoundationalDiagnosticOutcomeKind::Deferred,
                vec![required_row()],
                vec![],
                vec![],
                FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
                FoundationalDiagnosticPartiality::Complete,
                counters(),
                vec![FoundationalDiagnosticAssemblyDebt::new(
                    FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                    0,
                )],
            ),
            profile(
                DiagnosticRichnessProfile::Standard,
                SupportPostureProfile::SupportReady,
            ),
            FoundationalDiagnosticDeliveryClass::CanDefer,
        ),
        Err(FoundationalDiagnosticMaterializationDenial::RowScanFallbackMustRemainExplicitDebt)
    );
}
