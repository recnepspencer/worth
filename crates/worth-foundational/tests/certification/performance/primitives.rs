use worth_foundational::{
    evaluate_performance_primitive_legality, foundational_performance_boundary_definitions,
    foundational_performance_execution_temperature_definitions,
    foundational_performance_layout_intent_definitions, foundational_responsibilities,
    performance_api::common_path as performance_common, FoundationalPerformanceBoundary,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceLayoutIntent, FoundationalPerformancePrimitiveLegalityDenial,
    FoundationalPerformanceWorkClass,
};

#[test]
fn performance_responsibility_home_is_named_in_the_facade_topology() {
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
            "expressions",
        ]
    );
}

#[test]
fn primitive_family_definitions_are_blind_consumer_interpretable() {
    let layouts = foundational_performance_layout_intent_definitions();
    let layout_names: Vec<_> = layouts.iter().map(|definition| definition.name()).collect();
    assert_eq!(
        layout_names,
        vec!["aos", "soa", "aosoa", "sparse", "packed", "custom"]
    );
    assert!(layouts
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
    assert!(layouts
        .iter()
        .all(|definition| !definition.must_not_mean().trim().is_empty()));

    let boundaries = foundational_performance_boundary_definitions();
    let boundary_names: Vec<_> = boundaries
        .iter()
        .map(|definition| definition.name())
        .collect();
    assert_eq!(
        boundary_names,
        vec![
            "authoritative_execution",
            "boundary_materialization",
            "replay_reconstruction",
            "support_assembly",
            "maintenance_planning",
            "maintenance_execution",
            "publication",
            "delivery",
            "retention_compaction",
            "restore_recovery",
        ]
    );
}

#[test]
fn primitive_families_preserve_deterministic_ordering() {
    let mut layouts = vec![
        FoundationalPerformanceLayoutIntent::Packed,
        FoundationalPerformanceLayoutIntent::AoS,
        FoundationalPerformanceLayoutIntent::Custom,
    ];
    layouts.sort();
    assert_eq!(
        layouts,
        vec![
            FoundationalPerformanceLayoutIntent::AoS,
            FoundationalPerformanceLayoutIntent::Packed,
            FoundationalPerformanceLayoutIntent::Custom,
        ]
    );

    let mut temperatures = vec![
        FoundationalPerformanceExecutionTemperature::SupportOnly,
        FoundationalPerformanceExecutionTemperature::HotPath,
        FoundationalPerformanceExecutionTemperature::ColdPath,
    ];
    temperatures.sort();
    assert_eq!(
        temperatures,
        vec![
            FoundationalPerformanceExecutionTemperature::HotPath,
            FoundationalPerformanceExecutionTemperature::ColdPath,
            FoundationalPerformanceExecutionTemperature::SupportOnly,
        ]
    );

    let front_door = performance_common::performance();
    assert_eq!(
        front_door.execution_temperature_definitions(),
        foundational_performance_execution_temperature_definitions()
    );
}

#[test]
fn primitive_legality_floor_rejects_obvious_hot_path_and_debt_contradictions() {
    let hot_path = [FoundationalPerformanceWorkClass::AuthoritativeMutation];
    let hot_excluded = [
        FoundationalPerformanceWorkClass::ReplayReconstruction,
        FoundationalPerformanceWorkClass::SupportReportAssembly,
    ];

    assert_eq!(
        evaluate_performance_primitive_legality(
            FoundationalPerformanceBoundary::AuthoritativeExecution,
            FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim,
            FoundationalPerformanceExecutionTemperature::HotPath,
            FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            FoundationalPerformanceFallbackDebtPosture::Debt,
            &hot_path,
            &hot_excluded,
        ),
        Err(
            FoundationalPerformancePrimitiveLegalityDenial::SupportDerivedClaimsCannotUseHotPathTemperature
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            FoundationalPerformanceBoundary::AuthoritativeExecution,
            FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
            FoundationalPerformanceExecutionTemperature::WarmPath,
            FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            FoundationalPerformanceFallbackDebtPosture::Verified,
            &hot_path,
            &[],
        ),
        Err(
            FoundationalPerformancePrimitiveLegalityDenial::DebtOrDeferredEvidenceCannotClaimVerifiedPosture
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            FoundationalPerformanceBoundary::BoundaryMaterialization,
            FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission,
            FoundationalPerformanceExecutionTemperature::HotPath,
            FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            FoundationalPerformanceFallbackDebtPosture::Deferred,
            &hot_path,
            &hot_excluded,
        ),
        Err(
            FoundationalPerformancePrimitiveLegalityDenial::BoundaryMaterializationCannotClaimHotPathTemperature
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            FoundationalPerformanceBoundary::AuthoritativeExecution,
            FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
            FoundationalPerformanceExecutionTemperature::HotPath,
            FoundationalPerformanceFreshnessRetentionPosture::StaleSupport,
            FoundationalPerformanceFallbackDebtPosture::Verified,
            &hot_path,
            &hot_excluded,
        ),
        Err(
            FoundationalPerformancePrimitiveLegalityDenial::HotPathClaimsCannotUseReplayOrStaleFreshness
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            FoundationalPerformanceBoundary::AuthoritativeExecution,
            FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
            FoundationalPerformanceExecutionTemperature::HotPath,
            FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            FoundationalPerformanceFallbackDebtPosture::Verified,
            &[
                FoundationalPerformanceWorkClass::AuthoritativeMutation,
                FoundationalPerformanceWorkClass::ReplayReconstruction,
            ],
            &hot_excluded,
        ),
        Err(
            FoundationalPerformancePrimitiveLegalityDenial::HotPathIncludedWorkMustExcludeReplaySupportAndForensics
        )
    );
}

#[test]
fn front_door_keeps_legality_and_definition_surfaces_in_the_common_path() {
    let front_door = performance_common::performance();
    let result = front_door.evaluate_primitive_legality(
        FoundationalPerformanceBoundary::AuthoritativeExecution,
        FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        FoundationalPerformanceExecutionTemperature::HotPath,
        FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        FoundationalPerformanceFallbackDebtPosture::Verified,
        &[FoundationalPerformanceWorkClass::AuthoritativeMutation],
        &[
            FoundationalPerformanceWorkClass::ReplayReconstruction,
            FoundationalPerformanceWorkClass::SupportReportAssembly,
        ],
    );

    assert_eq!(result, Ok(()));
}
