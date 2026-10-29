use worth_foundational::{
    foundational_diagnostic_boundary_artifact_subject, foundational_diagnostic_code,
    foundational_diagnostic_locator_boundary_artifact, foundational_diagnostic_scope,
    AdmissionReadinessProfile, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalDiagnosticAssemblyDebt, FoundationalDiagnosticAssemblyDebtClass,
    FoundationalDiagnosticCounterSnapshot, FoundationalDiagnosticExplanationInput,
    FoundationalDiagnosticLocalityClaim, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticPartiality, FoundationalDiagnosticRow,
    FoundationalDiagnosticSemanticLabelSet, FoundationalDiagnosticSeverity,
    FoundationalDiagnosticSupportClaimStrength, FoundationalDiagnosticSupportEvidencePosture,
    FoundationalDiagnosticSupportInput, FoundationalDiagnosticSupportRow,
    FoundationalDiagnosticSurfaceAvailability, FoundationalDiagnosticWidenedFalloutPosture,
    FoundationalProfileSet, FoundationalProfileSetInput, RetentionDeliveryProfile,
    SupportPostureProfile,
};

pub fn profile(
    diagnostic_richness: DiagnosticRichnessProfile,
    support_posture: SupportPostureProfile,
) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness,
        support_posture,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("valid profile")
}

pub fn explanation_input(
    partiality: FoundationalDiagnosticPartiality,
) -> FoundationalDiagnosticExplanationInput {
    FoundationalDiagnosticExplanationInput::new(
        subject(),
        FoundationalDiagnosticOutcomeKind::Advisory,
        vec![required_row()],
        vec![standard_row()],
        vec![forensic_row()],
        FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
        partiality,
        counters(),
        vec![
            FoundationalDiagnosticAssemblyDebt::new(
                FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                2,
            ),
            FoundationalDiagnosticAssemblyDebt::new(
                FoundationalDiagnosticAssemblyDebtClass::RepeatedRediscovery,
                1,
            ),
        ],
    )
}

pub fn support_input(
    strength: FoundationalDiagnosticSupportClaimStrength,
    partiality: FoundationalDiagnosticPartiality,
    debts: Vec<FoundationalDiagnosticAssemblyDebt>,
) -> FoundationalDiagnosticSupportInput {
    FoundationalDiagnosticSupportInput::new(
        subject(),
        FoundationalDiagnosticOutcomeKind::Partial,
        vec![required_row()],
        vec![standard_row()],
        vec![],
        FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
        strength,
        partiality,
        counters(),
        debts,
    )
}

pub fn subject() -> worth_foundational::FoundationalDiagnosticSubject {
    foundational_diagnostic_boundary_artifact_subject(
        BoundaryArtifactId::new(51),
        BoundaryArtifactField::Payload,
    )
}

pub fn locator() -> worth_foundational::FoundationalDiagnosticLocator {
    foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(51),
        BoundaryArtifactField::Payload,
    ))
}

pub fn required_row() -> FoundationalDiagnosticRow {
    FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
        code("support.required"),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Advisory,
        subject(),
        locator(),
        FoundationalDiagnosticOutcomeKind::Accepted,
        labels(["required"]),
        FoundationalDiagnosticSupportEvidencePosture::Present(
            worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
        ),
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    ))
}

pub fn standard_row() -> FoundationalDiagnosticRow {
    FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
        code("support.standard"),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Info,
        subject(),
        locator(),
        FoundationalDiagnosticOutcomeKind::Advisory,
        labels(["standard"]),
        FoundationalDiagnosticSupportEvidencePosture::Present(
            worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized,
        ),
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    ))
}

pub fn forensic_row() -> FoundationalDiagnosticRow {
    FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
        code("support.forensic"),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Info,
        subject(),
        locator(),
        FoundationalDiagnosticOutcomeKind::Advisory,
        labels(["forensic"]),
        FoundationalDiagnosticSupportEvidencePosture::Present(
            worth_foundational::FoundationalDiagnosticEvidencePosture::Reconstructed,
        ),
        FoundationalDiagnosticLocalityClaim::SubjectNeighborhood,
        FoundationalDiagnosticWidenedFalloutPosture::WidenedExpected,
    ))
}

pub fn row_codes(rows: &[FoundationalDiagnosticRow]) -> Vec<&str> {
    rows.iter().map(|row| row.code().as_str()).collect()
}

pub fn counters() -> FoundationalDiagnosticCounterSnapshot {
    FoundationalDiagnosticCounterSnapshot::new(2, 1, 0, 2, 0, 1)
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
