use worth_foundational::{
    foundational_diagnostic_code, foundational_diagnostic_locator_boundary_artifact,
    foundational_diagnostic_scope, BoundaryArtifactField, BoundaryArtifactId,
    BoundaryArtifactLocator, CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind,
    CanonicalBasisLocus, CanonicalBasisValue, CanonicalIntegerWidth, DiagnosticRichnessProfile,
    FoundationalDiagnosticComparisonRow, FoundationalDiagnosticDecisionRow,
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticEvidencePosture,
    FoundationalDiagnosticExplanationBundle, FoundationalDiagnosticExplanationInput,
    FoundationalDiagnosticGapClass, FoundationalDiagnosticGapClosurePosture,
    FoundationalDiagnosticGapTarget, FoundationalDiagnosticLocalityClaim,
    FoundationalDiagnosticNamedGap, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticPartiality, FoundationalDiagnosticProvenanceReadyRow,
    FoundationalDiagnosticRow, FoundationalDiagnosticSemanticLabelSet,
    FoundationalDiagnosticSeverity, FoundationalDiagnosticSupportClaimStrength,
    FoundationalDiagnosticSupportEvidencePosture, FoundationalDiagnosticSupportInput,
    FoundationalDiagnosticSupportReport, FoundationalDiagnosticSupportRow,
    FoundationalDiagnosticSurfaceAvailability, FoundationalDiagnosticWidenedFalloutPosture,
    SupportPostureProfile,
};

use super::materialization_support::{counters, locator, profile, subject};

pub fn version() -> worth_foundational::CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("milestone-6-phase-4")
        .expect("valid version")
}

pub fn support_report_with_unsorted_inputs() -> FoundationalDiagnosticSupportReport {
    worth_foundational::materialize_diagnostic_support_report(
        FoundationalDiagnosticSupportInput::new(
            subject(),
            FoundationalDiagnosticOutcomeKind::Partial,
            vec![support_row("support.required", FoundationalDiagnosticEvidencePosture::RetainedDirect)],
            vec![support_row("support.standard", FoundationalDiagnosticEvidencePosture::Summarized)],
            vec![],
            FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                FoundationalDiagnosticNamedGap::new(
                    FoundationalDiagnosticGapClass::WidenedFallback,
                    FoundationalDiagnosticGapTarget::Subject(subject()),
                    FoundationalDiagnosticGapClosurePosture::Deferred,
                ),
                FoundationalDiagnosticNamedGap::new(
                    FoundationalDiagnosticGapClass::CoverageOmission,
                    FoundationalDiagnosticGapTarget::Locator(locator()),
                    FoundationalDiagnosticGapClosurePosture::DebtNamed,
                ),
            ]),
            counters(),
            vec![
                worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                    worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RepeatedRediscovery,
                    1,
                ),
                worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                    worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                    2,
                ),
            ],
        ),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("support report")
}

pub fn support_report_equivalent_reordered() -> FoundationalDiagnosticSupportReport {
    worth_foundational::materialize_diagnostic_support_report(
        FoundationalDiagnosticSupportInput::new(
            subject(),
            FoundationalDiagnosticOutcomeKind::Partial,
            vec![support_row("support.required", FoundationalDiagnosticEvidencePosture::RetainedDirect)],
            vec![support_row("support.standard", FoundationalDiagnosticEvidencePosture::Summarized)],
            vec![],
            FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                FoundationalDiagnosticNamedGap::new(
                    FoundationalDiagnosticGapClass::CoverageOmission,
                    FoundationalDiagnosticGapTarget::Locator(locator()),
                    FoundationalDiagnosticGapClosurePosture::DebtNamed,
                ),
                FoundationalDiagnosticNamedGap::new(
                    FoundationalDiagnosticGapClass::WidenedFallback,
                    FoundationalDiagnosticGapTarget::Subject(subject()),
                    FoundationalDiagnosticGapClosurePosture::Deferred,
                ),
            ]),
            counters(),
            vec![
                worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                    worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                    2,
                ),
                worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                    worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RepeatedRediscovery,
                    1,
                ),
            ],
        ),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("equivalent support report")
}

pub fn explanation_bundle_with_mixed_rows(
    comparison_posture: FoundationalDiagnosticEvidencePosture,
) -> FoundationalDiagnosticExplanationBundle {
    worth_foundational::materialize_diagnostic_explanation_bundle(
        FoundationalDiagnosticExplanationInput::new(
            subject(),
            FoundationalDiagnosticOutcomeKind::Mismatch,
            vec![
                FoundationalDiagnosticRow::Decision(FoundationalDiagnosticDecisionRow::new(
                    code("decision.branch"),
                    scope("diagnostics.decision"),
                    FoundationalDiagnosticSeverity::Info,
                    subject(),
                    locator(),
                    FoundationalDiagnosticOutcomeKind::Accepted,
                    labels(["decision"]),
                    None,
                    FoundationalDiagnosticLocalityClaim::ExactSubject,
                    FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
                )),
                support_row(
                    "support.required",
                    FoundationalDiagnosticEvidencePosture::RetainedDirect,
                ),
            ],
            vec![FoundationalDiagnosticRow::Comparison(
                FoundationalDiagnosticComparisonRow::new(
                    code("comparison.parity"),
                    scope("diagnostics.comparison"),
                    FoundationalDiagnosticSeverity::Advisory,
                    subject(),
                    locator(),
                    FoundationalDiagnosticOutcomeKind::Mismatch,
                    labels(["comparison"]),
                    Some(locator()),
                    comparison_posture,
                ),
            )],
            vec![FoundationalDiagnosticRow::ProvenanceReady(
                FoundationalDiagnosticProvenanceReadyRow::new(
                    code("provenance.origin"),
                    scope("diagnostics.provenance"),
                    FoundationalDiagnosticSeverity::Info,
                    subject(),
                    locator(),
                    FoundationalDiagnosticOutcomeKind::Deferred,
                    labels(["provenance"]),
                    foundational_diagnostic_locator_boundary_artifact(
                        BoundaryArtifactLocator::new(
                            BoundaryArtifactId::new(51),
                            BoundaryArtifactField::Proofs,
                        ),
                    ),
                    FoundationalDiagnosticEvidencePosture::Reconstructed,
                ),
            )],
            FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                FoundationalDiagnosticNamedGap::new(
                    FoundationalDiagnosticGapClass::CoverageOmission,
                    FoundationalDiagnosticGapTarget::Locator(locator()),
                    FoundationalDiagnosticGapClosurePosture::DebtNamed,
                ),
            ]),
            counters(),
            vec![worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                2,
            )],
        ),
        profile(
            DiagnosticRichnessProfile::Forensic,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation bundle")
}

pub fn explanation_bundle_with_tied_common_rows(
    comparison_rows_reversed: bool,
) -> FoundationalDiagnosticExplanationBundle {
    let first = FoundationalDiagnosticRow::Comparison(FoundationalDiagnosticComparisonRow::new(
        code("comparison.tied"),
        scope("diagnostics.comparison"),
        FoundationalDiagnosticSeverity::Advisory,
        subject(),
        locator(),
        FoundationalDiagnosticOutcomeKind::Mismatch,
        labels(["comparison", "tied"]),
        None,
        FoundationalDiagnosticEvidencePosture::RetainedDirect,
    ));
    let second = FoundationalDiagnosticRow::Comparison(FoundationalDiagnosticComparisonRow::new(
        code("comparison.tied"),
        scope("diagnostics.comparison"),
        FoundationalDiagnosticSeverity::Advisory,
        subject(),
        locator(),
        FoundationalDiagnosticOutcomeKind::Mismatch,
        labels(["comparison", "tied"]),
        Some(locator()),
        FoundationalDiagnosticEvidencePosture::Summarized,
    ));
    let standard_rows = if comparison_rows_reversed {
        vec![second, first]
    } else {
        vec![first, second]
    };

    worth_foundational::materialize_diagnostic_explanation_bundle(
        FoundationalDiagnosticExplanationInput::new(
            subject(),
            FoundationalDiagnosticOutcomeKind::Mismatch,
            vec![],
            standard_rows,
            vec![],
            FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            FoundationalDiagnosticPartiality::Complete,
            counters(),
            vec![],
        ),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation bundle with tied common rows")
}

pub fn diagnostic_text_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: &str,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Diagnostic,
        CanonicalBasisLocus::Named(locus.into()),
        kind,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

pub fn diagnostic_integer_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: u64,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Diagnostic,
        CanonicalBasisLocus::Named(locus.into()),
        kind,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

pub fn diagnostic_bool_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: bool,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Diagnostic,
        CanonicalBasisLocus::Named(locus.into()),
        kind,
        CanonicalBasisValue::Bool(value),
    )
}

fn support_row(
    code_value: &str,
    posture: FoundationalDiagnosticEvidencePosture,
) -> FoundationalDiagnosticRow {
    FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
        code(code_value),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Advisory,
        subject(),
        locator(),
        FoundationalDiagnosticOutcomeKind::Accepted,
        labels([code_value]),
        FoundationalDiagnosticSupportEvidencePosture::Present(posture),
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    ))
}

fn code(value: &str) -> worth_foundational::FoundationalDiagnosticCodeId {
    foundational_diagnostic_code(value).expect("valid code")
}

fn scope(value: &str) -> worth_foundational::FoundationalDiagnosticScopeId {
    foundational_diagnostic_scope(value).expect("valid scope")
}

fn labels<const N: usize>(values: [&str; N]) -> FoundationalDiagnosticSemanticLabelSet {
    FoundationalDiagnosticSemanticLabelSet::new(values.into_iter().map(code))
}
