pub use {
    worth_foundational::foundational_diagnostic_code,
    worth_foundational::foundational_diagnostic_locator_boundary_artifact,
    worth_foundational::foundational_diagnostic_scope, worth_foundational::BoundaryArtifactField,
    worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator,
    worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry,
    worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalIntegerWidth,
    worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalDiagnosticComparisonRow,
    worth_foundational::FoundationalDiagnosticDecisionRow,
    worth_foundational::FoundationalDiagnosticDeliveryClass,
    worth_foundational::FoundationalDiagnosticEvidencePosture,
    worth_foundational::FoundationalDiagnosticExplanationBundle,
    worth_foundational::FoundationalDiagnosticExplanationInput,
    worth_foundational::FoundationalDiagnosticGapClass,
    worth_foundational::FoundationalDiagnosticGapClosurePosture,
    worth_foundational::FoundationalDiagnosticGapTarget,
    worth_foundational::FoundationalDiagnosticLocalityClaim,
    worth_foundational::FoundationalDiagnosticNamedGap,
    worth_foundational::FoundationalDiagnosticOutcomeKind,
    worth_foundational::FoundationalDiagnosticPartiality,
    worth_foundational::FoundationalDiagnosticProvenanceReadyRow,
    worth_foundational::FoundationalDiagnosticRow,
    worth_foundational::FoundationalDiagnosticSemanticLabelSet,
    worth_foundational::FoundationalDiagnosticSeverity,
    worth_foundational::FoundationalDiagnosticSupportClaimStrength,
    worth_foundational::FoundationalDiagnosticSupportEvidencePosture,
    worth_foundational::FoundationalDiagnosticSupportInput,
    worth_foundational::FoundationalDiagnosticSupportReport,
    worth_foundational::FoundationalDiagnosticSupportRow,
    worth_foundational::FoundationalDiagnosticSurfaceAvailability,
    worth_foundational::FoundationalDiagnosticWidenedFalloutPosture,
    worth_foundational::SupportPostureProfile,
};

pub use {
    crate::certification::ordinary_context::diagnostics::materialization_support::counters,
    crate::certification::ordinary_context::diagnostics::materialization_support::locator,
    crate::certification::ordinary_context::diagnostics::materialization_support::profile,
    crate::certification::ordinary_context::diagnostics::materialization_support::subject,
};

pub fn version() -> worth_foundational::CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("milestone-6-phase-4")
        .expect("valid version")
}

pub fn support_report_with_unsorted_inputs() -> FoundationalDiagnosticSupportReport {
    worth_foundational::materialize_diagnostic_support_report(
        worth_foundational::FoundationalDiagnosticSupportInput::new(
            subject(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Partial,
            vec![support_row("support.required", worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect)],
            vec![support_row("support.standard", worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized)],
            vec![],
            worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            worth_foundational::FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            worth_foundational::FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                worth_foundational::FoundationalDiagnosticNamedGap::new(
                    worth_foundational::FoundationalDiagnosticGapClass::WidenedFallback,
                    worth_foundational::FoundationalDiagnosticGapTarget::Subject(subject()),
                    worth_foundational::FoundationalDiagnosticGapClosurePosture::Deferred,
                ),
                worth_foundational::FoundationalDiagnosticNamedGap::new(
                    worth_foundational::FoundationalDiagnosticGapClass::CoverageOmission,
                    worth_foundational::FoundationalDiagnosticGapTarget::Locator(locator()),
                    worth_foundational::FoundationalDiagnosticGapClosurePosture::DebtNamed,
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
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("support report")
}

pub fn support_report_equivalent_reordered() -> FoundationalDiagnosticSupportReport {
    worth_foundational::materialize_diagnostic_support_report(
        worth_foundational::FoundationalDiagnosticSupportInput::new(
            subject(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Partial,
            vec![support_row("support.required", worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect)],
            vec![support_row("support.standard", worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized)],
            vec![],
            worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            worth_foundational::FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            worth_foundational::FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                worth_foundational::FoundationalDiagnosticNamedGap::new(
                    worth_foundational::FoundationalDiagnosticGapClass::CoverageOmission,
                    worth_foundational::FoundationalDiagnosticGapTarget::Locator(locator()),
                    worth_foundational::FoundationalDiagnosticGapClosurePosture::DebtNamed,
                ),
                worth_foundational::FoundationalDiagnosticNamedGap::new(
                    worth_foundational::FoundationalDiagnosticGapClass::WidenedFallback,
                    worth_foundational::FoundationalDiagnosticGapTarget::Subject(subject()),
                    worth_foundational::FoundationalDiagnosticGapClosurePosture::Deferred,
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
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("equivalent support report")
}

pub fn explanation_bundle_with_mixed_rows(
    comparison_posture: FoundationalDiagnosticEvidencePosture,
) -> FoundationalDiagnosticExplanationBundle {
    worth_foundational::materialize_diagnostic_explanation_bundle(
        worth_foundational::FoundationalDiagnosticExplanationInput::new(
            subject(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
            vec![
                worth_foundational::FoundationalDiagnosticRow::Decision(
                    worth_foundational::FoundationalDiagnosticDecisionRow::new(
                        code("decision.branch"),
                        scope("diagnostics.decision"),
                        worth_foundational::FoundationalDiagnosticSeverity::Info,
                        subject(),
                        locator(),
                        worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
                        labels(["decision"]),
                        None,
                        worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
                        worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
                    ),
                ),
                support_row(
                    "support.required",
                    worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
                ),
            ],
            vec![worth_foundational::FoundationalDiagnosticRow::Comparison(
                worth_foundational::FoundationalDiagnosticComparisonRow::new(
                    code("comparison.parity"),
                    scope("diagnostics.comparison"),
                    worth_foundational::FoundationalDiagnosticSeverity::Advisory,
                    subject(),
                    locator(),
                    worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
                    labels(["comparison"]),
                    Some(locator()),
                    comparison_posture,
                ),
            )],
            vec![
                worth_foundational::FoundationalDiagnosticRow::ProvenanceReady(
                    worth_foundational::FoundationalDiagnosticProvenanceReadyRow::new(
                        code("provenance.origin"),
                        scope("diagnostics.provenance"),
                        worth_foundational::FoundationalDiagnosticSeverity::Info,
                        subject(),
                        locator(),
                        worth_foundational::FoundationalDiagnosticOutcomeKind::Deferred,
                        labels(["provenance"]),
                        foundational_diagnostic_locator_boundary_artifact(
                            worth_foundational::BoundaryArtifactLocator::new(
                                worth_foundational::BoundaryArtifactId::new(51),
                                worth_foundational::BoundaryArtifactField::Proofs,
                            ),
                        ),
                        worth_foundational::FoundationalDiagnosticEvidencePosture::Reconstructed,
                    ),
                ),
            ],
            worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            worth_foundational::FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                worth_foundational::FoundationalDiagnosticNamedGap::new(
                    worth_foundational::FoundationalDiagnosticGapClass::CoverageOmission,
                    worth_foundational::FoundationalDiagnosticGapTarget::Locator(locator()),
                    worth_foundational::FoundationalDiagnosticGapClosurePosture::DebtNamed,
                ),
            ]),
            counters(),
            vec![worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                2,
            )],
        ),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Forensic,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation bundle")
}

pub fn explanation_bundle_with_tied_common_rows(
    comparison_rows_reversed: bool,
) -> FoundationalDiagnosticExplanationBundle {
    let first = worth_foundational::FoundationalDiagnosticRow::Comparison(
        worth_foundational::FoundationalDiagnosticComparisonRow::new(
            code("comparison.tied"),
            scope("diagnostics.comparison"),
            worth_foundational::FoundationalDiagnosticSeverity::Advisory,
            subject(),
            locator(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
            labels(["comparison", "tied"]),
            None,
            worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
        ),
    );
    let second = worth_foundational::FoundationalDiagnosticRow::Comparison(
        worth_foundational::FoundationalDiagnosticComparisonRow::new(
            code("comparison.tied"),
            scope("diagnostics.comparison"),
            worth_foundational::FoundationalDiagnosticSeverity::Advisory,
            subject(),
            locator(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
            labels(["comparison", "tied"]),
            Some(locator()),
            worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized,
        ),
    );
    let standard_rows = if comparison_rows_reversed {
        vec![second, first]
    } else {
        vec![first, second]
    };

    worth_foundational::materialize_diagnostic_explanation_bundle(
        worth_foundational::FoundationalDiagnosticExplanationInput::new(
            subject(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
            vec![],
            standard_rows,
            vec![],
            worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
            worth_foundational::FoundationalDiagnosticPartiality::Complete,
            counters(),
            vec![],
        ),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
        ),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation bundle with tied common rows")
}

pub fn diagnostic_text_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: &str,
) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Diagnostic,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        kind,
        worth_foundational::CanonicalBasisValue::ExactText(value.into()),
    )
}

pub fn diagnostic_integer_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: u64,
) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Diagnostic,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        kind,
        worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

pub fn diagnostic_bool_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: bool,
) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Diagnostic,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        kind,
        worth_foundational::CanonicalBasisValue::Bool(value),
    )
}

pub fn support_row(
    code_value: &str,
    posture: FoundationalDiagnosticEvidencePosture,
) -> FoundationalDiagnosticRow {
    worth_foundational::FoundationalDiagnosticRow::Support(
        worth_foundational::FoundationalDiagnosticSupportRow::new(
            code(code_value),
            scope("diagnostics.support"),
            worth_foundational::FoundationalDiagnosticSeverity::Advisory,
            subject(),
            locator(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
            labels([code_value]),
            worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(posture),
            worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
            worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
        ),
    )
}

pub fn code(value: &str) -> worth_foundational::FoundationalDiagnosticCodeId {
    foundational_diagnostic_code(value).expect("valid code")
}

pub fn scope(value: &str) -> worth_foundational::FoundationalDiagnosticScopeId {
    foundational_diagnostic_scope(value).expect("valid scope")
}

pub fn labels<const N: usize>(values: [&str; N]) -> FoundationalDiagnosticSemanticLabelSet {
    worth_foundational::FoundationalDiagnosticSemanticLabelSet::new(values.into_iter().map(code))
}
