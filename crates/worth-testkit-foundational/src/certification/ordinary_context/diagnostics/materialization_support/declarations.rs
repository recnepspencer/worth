pub use {
    worth_foundational::foundational_diagnostic_boundary_artifact_subject,
    worth_foundational::foundational_diagnostic_code,
    worth_foundational::foundational_diagnostic_locator_boundary_artifact,
    worth_foundational::foundational_diagnostic_scope,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::BoundaryArtifactField,
    worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalDiagnosticAssemblyDebt,
    worth_foundational::FoundationalDiagnosticAssemblyDebtClass,
    worth_foundational::FoundationalDiagnosticCounterSnapshot,
    worth_foundational::FoundationalDiagnosticExplanationInput,
    worth_foundational::FoundationalDiagnosticLocalityClaim,
    worth_foundational::FoundationalDiagnosticOutcomeKind,
    worth_foundational::FoundationalDiagnosticPartiality,
    worth_foundational::FoundationalDiagnosticRow,
    worth_foundational::FoundationalDiagnosticSemanticLabelSet,
    worth_foundational::FoundationalDiagnosticSeverity,
    worth_foundational::FoundationalDiagnosticSupportClaimStrength,
    worth_foundational::FoundationalDiagnosticSupportEvidencePosture,
    worth_foundational::FoundationalDiagnosticSupportInput,
    worth_foundational::FoundationalDiagnosticSupportRow,
    worth_foundational::FoundationalDiagnosticSurfaceAvailability,
    worth_foundational::FoundationalDiagnosticWidenedFalloutPosture,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub fn profile(
    diagnostic_richness: DiagnosticRichnessProfile,
    support_posture: SupportPostureProfile,
) -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness,
        support_posture,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::Uncertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("valid profile")
}

pub fn explanation_input(
    partiality: FoundationalDiagnosticPartiality,
) -> FoundationalDiagnosticExplanationInput {
    worth_foundational::FoundationalDiagnosticExplanationInput::new(
        subject(),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Advisory,
        vec![required_row()],
        vec![standard_row()],
        vec![forensic_row()],
        worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
        partiality,
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
    )
}

pub fn support_input(
    strength: FoundationalDiagnosticSupportClaimStrength,
    partiality: FoundationalDiagnosticPartiality,
    debts: Vec<FoundationalDiagnosticAssemblyDebt>,
) -> FoundationalDiagnosticSupportInput {
    worth_foundational::FoundationalDiagnosticSupportInput::new(
        subject(),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Partial,
        vec![required_row()],
        vec![standard_row()],
        vec![],
        worth_foundational::FoundationalDiagnosticSurfaceAvailability::deferred_cold(),
        strength,
        partiality,
        counters(),
        debts,
    )
}

pub fn subject() -> worth_foundational::FoundationalDiagnosticSubject {
    foundational_diagnostic_boundary_artifact_subject(
        worth_foundational::BoundaryArtifactId::new(51),
        worth_foundational::BoundaryArtifactField::Payload,
    )
}

pub fn locator() -> worth_foundational::FoundationalDiagnosticLocator {
    foundational_diagnostic_locator_boundary_artifact(
        worth_foundational::BoundaryArtifactLocator::new(
            worth_foundational::BoundaryArtifactId::new(51),
            worth_foundational::BoundaryArtifactField::Payload,
        ),
    )
}

pub fn required_row() -> FoundationalDiagnosticRow {
    worth_foundational::FoundationalDiagnosticRow::Support(
        worth_foundational::FoundationalDiagnosticSupportRow::new(
            code("support.required"),
            scope("diagnostics.support"),
            worth_foundational::FoundationalDiagnosticSeverity::Advisory,
            subject(),
            locator(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
            labels(["required"]),
            worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(
                worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
            ),
            worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
            worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
        ),
    )
}

pub fn standard_row() -> FoundationalDiagnosticRow {
    worth_foundational::FoundationalDiagnosticRow::Support(
        worth_foundational::FoundationalDiagnosticSupportRow::new(
            code("support.standard"),
            scope("diagnostics.support"),
            worth_foundational::FoundationalDiagnosticSeverity::Info,
            subject(),
            locator(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Advisory,
            labels(["standard"]),
            worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(
                worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized,
            ),
            worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
            worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
        ),
    )
}

pub fn forensic_row() -> FoundationalDiagnosticRow {
    worth_foundational::FoundationalDiagnosticRow::Support(
        worth_foundational::FoundationalDiagnosticSupportRow::new(
            code("support.forensic"),
            scope("diagnostics.support"),
            worth_foundational::FoundationalDiagnosticSeverity::Info,
            subject(),
            locator(),
            worth_foundational::FoundationalDiagnosticOutcomeKind::Advisory,
            labels(["forensic"]),
            worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(
                worth_foundational::FoundationalDiagnosticEvidencePosture::Reconstructed,
            ),
            worth_foundational::FoundationalDiagnosticLocalityClaim::SubjectNeighborhood,
            worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::WidenedExpected,
        ),
    )
}

pub fn row_codes(rows: &[FoundationalDiagnosticRow]) -> Vec<&str> {
    rows.iter().map(|row| row.code().as_str()).collect()
}

pub fn counters() -> FoundationalDiagnosticCounterSnapshot {
    worth_foundational::FoundationalDiagnosticCounterSnapshot::new(2, 1, 0, 2, 0, 1)
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
