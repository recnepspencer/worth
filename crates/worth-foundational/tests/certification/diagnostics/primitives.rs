use worth_foundational::{
    diagnostic_artifact_kind_definitions, evaluate_diagnostic_materialization_legality,
    foundational_diagnostic_code, foundational_diagnostic_scope, foundational_responsibilities,
    FoundationalDiagnosticArtifactKind, FoundationalDiagnosticAvailability,
    FoundationalDiagnosticBreachClass, FoundationalDiagnosticDeliveryClass,
    FoundationalDiagnosticEvidencePosture, FoundationalDiagnosticMaterializationLegalityDenial,
    FoundationalDiagnosticPrimitiveConstructionDenial, FoundationalDiagnosticSeverity,
};

#[test]
fn diagnostics_responsibility_home_is_named_in_the_facade_topology() {
    let names: Vec<_> = foundational_responsibilities()
        .iter()
        .map(|area| area.name())
        .collect();

    assert_eq!(
        names,
        vec![
            "canonical_values",
            "aspect_state_and_patches",
            "identity_categories",
            "locators",
            "compatibility_bridges",
            "canonical_ordering_and_equality",
            "profiles",
            "boundary_artifacts",
            "transitions",
            "diagnostics",
            "boundary_evidence",
            "performance",
            "budget_limits",
            "expressions",
        ]
    );
}

#[test]
fn diagnostic_primitive_ids_require_canonical_token_labels() {
    let code = foundational_diagnostic_code("merge.conflict").expect("valid code");
    let scope = foundational_diagnostic_scope("transition.merge").expect("valid scope");

    assert_eq!(code.as_str(), "merge.conflict");
    assert_eq!(scope.as_str(), "transition.merge");

    assert_eq!(
        foundational_diagnostic_code(""),
        Err(FoundationalDiagnosticPrimitiveConstructionDenial::CodeMustNotBeEmpty)
    );
    assert_eq!(
        foundational_diagnostic_scope("Scope.Mixed"),
        Err(FoundationalDiagnosticPrimitiveConstructionDenial::ScopeMustUseLowercaseAsciiTokens)
    );
    assert_eq!(
        foundational_diagnostic_code("merge..conflict"),
        Err(FoundationalDiagnosticPrimitiveConstructionDenial::CodeMustNotContainEmptySegments)
    );
    assert_eq!(
        foundational_diagnostic_scope("transition.merge."),
        Err(FoundationalDiagnosticPrimitiveConstructionDenial::ScopeMustNotContainEmptySegments)
    );
}

#[test]
fn primitive_families_preserve_deterministic_ordering() {
    let mut severities = vec![
        FoundationalDiagnosticSeverity::Violation,
        FoundationalDiagnosticSeverity::Advisory,
        FoundationalDiagnosticSeverity::Failure,
        FoundationalDiagnosticSeverity::Info,
    ];
    severities.sort();
    assert_eq!(
        severities,
        vec![
            FoundationalDiagnosticSeverity::Info,
            FoundationalDiagnosticSeverity::Advisory,
            FoundationalDiagnosticSeverity::Failure,
            FoundationalDiagnosticSeverity::Violation,
        ]
    );

    let mut postures = vec![
        FoundationalDiagnosticEvidencePosture::Redacted,
        FoundationalDiagnosticEvidencePosture::RetainedDirect,
        FoundationalDiagnosticEvidencePosture::AbsentExpected,
    ];
    postures.sort();
    assert_eq!(
        postures,
        vec![
            FoundationalDiagnosticEvidencePosture::RetainedDirect,
            FoundationalDiagnosticEvidencePosture::Redacted,
            FoundationalDiagnosticEvidencePosture::AbsentExpected,
        ]
    );

    let mut breaches = vec![
        FoundationalDiagnosticBreachClass::CoverageOmission,
        FoundationalDiagnosticBreachClass::ConstructionBug,
    ];
    breaches.sort();
    assert_eq!(
        breaches,
        vec![
            FoundationalDiagnosticBreachClass::ConstructionBug,
            FoundationalDiagnosticBreachClass::CoverageOmission,
        ]
    );

    let mut denials = vec![
        worth_foundational::FoundationalDiagnosticDenialClass::UnsupportedDenied,
        worth_foundational::FoundationalDiagnosticDenialClass::DomainDenied,
    ];
    denials.sort();
    assert_eq!(
        denials,
        vec![
            worth_foundational::FoundationalDiagnosticDenialClass::DomainDenied,
            worth_foundational::FoundationalDiagnosticDenialClass::UnsupportedDenied,
        ]
    );
}

#[test]
fn artifact_kind_definitions_are_blind_consumer_interpretable_and_canonically_ordered() {
    let definitions = diagnostic_artifact_kind_definitions();
    let names: Vec<_> = definitions
        .iter()
        .map(|definition| definition.name())
        .collect();

    assert_eq!(
        names,
        vec![
            "summary",
            "report",
            "failure_bundle",
            "comparison_bundle",
            "support_report",
            "explanation_bundle",
        ]
    );
    assert!(definitions
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
    assert!(definitions
        .iter()
        .all(|definition| !definition.must_not_mean().trim().is_empty()));
}

#[test]
fn materialization_legality_keeps_delivery_and_availability_explicit() {
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            FoundationalDiagnosticArtifactKind::Summary,
            FoundationalDiagnosticDeliveryClass::MustBeHot,
            FoundationalDiagnosticAvailability::DeferredCold,
        ),
        Err(
            FoundationalDiagnosticMaterializationLegalityDenial::MustBeHotRequiresRetainedHotAvailability
        )
    );
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            FoundationalDiagnosticArtifactKind::Summary,
            FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
            FoundationalDiagnosticAvailability::Reconstructable,
        ),
        Err(
            FoundationalDiagnosticMaterializationLegalityDenial::SummaryDoesNotSupportReplayReconstruction
        )
    );
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            FoundationalDiagnosticArtifactKind::Report,
            FoundationalDiagnosticDeliveryClass::UnavailableByPolicy,
            FoundationalDiagnosticAvailability::RetainedHot,
        ),
        Err(
            FoundationalDiagnosticMaterializationLegalityDenial::UnavailableByPolicyRequiresRedactedOrUnavailableAvailability
        )
    );

    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            FoundationalDiagnosticArtifactKind::SupportReport,
            FoundationalDiagnosticDeliveryClass::CanDefer,
            FoundationalDiagnosticAvailability::DeferredCold,
        ),
        Ok(())
    );
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            FoundationalDiagnosticArtifactKind::ExplanationBundle,
            FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
            FoundationalDiagnosticAvailability::Reconstructable,
        ),
        Ok(())
    );
}
