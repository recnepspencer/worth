use {worth_foundational::diagnostic_artifact_kind_definitions, worth_foundational::evaluate_diagnostic_materialization_legality, worth_foundational::foundational_diagnostic_code, worth_foundational::foundational_diagnostic_scope, worth_foundational::foundational_responsibilities, worth_foundational::FoundationalDiagnosticArtifactKind, worth_foundational::FoundationalDiagnosticAvailability, worth_foundational::FoundationalDiagnosticBreachClass, worth_foundational::FoundationalDiagnosticDeliveryClass, worth_foundational::FoundationalDiagnosticEvidencePosture, worth_foundational::FoundationalDiagnosticMaterializationLegalityDenial, worth_foundational::FoundationalDiagnosticPrimitiveConstructionDenial, worth_foundational::FoundationalDiagnosticSeverity};

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
        Err(worth_foundational::FoundationalDiagnosticPrimitiveConstructionDenial::CodeMustNotBeEmpty)
    );
    assert_eq!(
        foundational_diagnostic_scope("Scope.Mixed"),
        Err(worth_foundational::FoundationalDiagnosticPrimitiveConstructionDenial::ScopeMustUseLowercaseAsciiTokens)
    );
    assert_eq!(
        foundational_diagnostic_code("merge..conflict"),
        Err(worth_foundational::FoundationalDiagnosticPrimitiveConstructionDenial::CodeMustNotContainEmptySegments)
    );
    assert_eq!(
        foundational_diagnostic_scope("transition.merge."),
        Err(worth_foundational::FoundationalDiagnosticPrimitiveConstructionDenial::ScopeMustNotContainEmptySegments)
    );
}

#[test]
fn primitive_families_preserve_deterministic_ordering() {
    let mut severities = vec![
        worth_foundational::FoundationalDiagnosticSeverity::Violation,
        worth_foundational::FoundationalDiagnosticSeverity::Advisory,
        worth_foundational::FoundationalDiagnosticSeverity::Failure,
        worth_foundational::FoundationalDiagnosticSeverity::Info,
    ];
    severities.sort();
    assert_eq!(
        severities,
        vec![
            worth_foundational::FoundationalDiagnosticSeverity::Info,
            worth_foundational::FoundationalDiagnosticSeverity::Advisory,
            worth_foundational::FoundationalDiagnosticSeverity::Failure,
            worth_foundational::FoundationalDiagnosticSeverity::Violation,
        ]
    );

    let mut postures = vec![
        worth_foundational::FoundationalDiagnosticEvidencePosture::Redacted,
        worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
        worth_foundational::FoundationalDiagnosticEvidencePosture::AbsentExpected,
    ];
    postures.sort();
    assert_eq!(
        postures,
        vec![
            worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
            worth_foundational::FoundationalDiagnosticEvidencePosture::Redacted,
            worth_foundational::FoundationalDiagnosticEvidencePosture::AbsentExpected,
        ]
    );

    let mut breaches = vec![
        worth_foundational::FoundationalDiagnosticBreachClass::CoverageOmission,
        worth_foundational::FoundationalDiagnosticBreachClass::ConstructionBug,
    ];
    breaches.sort();
    assert_eq!(
        breaches,
        vec![
            worth_foundational::FoundationalDiagnosticBreachClass::ConstructionBug,
            worth_foundational::FoundationalDiagnosticBreachClass::CoverageOmission,
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
            worth_foundational::FoundationalDiagnosticArtifactKind::Summary,
            worth_foundational::FoundationalDiagnosticDeliveryClass::MustBeHot,
            worth_foundational::FoundationalDiagnosticAvailability::DeferredCold,
        ),
        Err(
            worth_foundational::FoundationalDiagnosticMaterializationLegalityDenial::MustBeHotRequiresRetainedHotAvailability
        )
    );
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            worth_foundational::FoundationalDiagnosticArtifactKind::Summary,
            worth_foundational::FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
            worth_foundational::FoundationalDiagnosticAvailability::Reconstructable,
        ),
        Err(
            worth_foundational::FoundationalDiagnosticMaterializationLegalityDenial::SummaryDoesNotSupportReplayReconstruction
        )
    );
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            worth_foundational::FoundationalDiagnosticArtifactKind::Report,
            worth_foundational::FoundationalDiagnosticDeliveryClass::UnavailableByPolicy,
            worth_foundational::FoundationalDiagnosticAvailability::RetainedHot,
        ),
        Err(
            worth_foundational::FoundationalDiagnosticMaterializationLegalityDenial::UnavailableByPolicyRequiresRedactedOrUnavailableAvailability
        )
    );

    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            worth_foundational::FoundationalDiagnosticArtifactKind::SupportReport,
            worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
            worth_foundational::FoundationalDiagnosticAvailability::DeferredCold,
        ),
        Ok(())
    );
    assert_eq!(
        evaluate_diagnostic_materialization_legality(
            worth_foundational::FoundationalDiagnosticArtifactKind::ExplanationBundle,
            worth_foundational::FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay,
            worth_foundational::FoundationalDiagnosticAvailability::Reconstructable,
        ),
        Ok(())
    );
}
