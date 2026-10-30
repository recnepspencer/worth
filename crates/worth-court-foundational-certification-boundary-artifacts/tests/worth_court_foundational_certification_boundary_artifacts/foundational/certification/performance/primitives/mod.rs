use {
    worth_foundational::evaluate_performance_primitive_legality,
    worth_foundational::foundational_performance_boundary_definitions,
    worth_foundational::foundational_performance_execution_temperature_definitions,
    worth_foundational::foundational_performance_layout_intent_definitions,
    worth_foundational::foundational_responsibilities,
    worth_foundational::performance_api::common_path as performance_common,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceLayoutIntent,
    worth_foundational::FoundationalPerformancePrimitiveLegalityDenial,
    worth_foundational::FoundationalPerformanceWorkClass,
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
            "budget_limits",
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
        worth_foundational::FoundationalPerformanceLayoutIntent::Packed,
        worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
        worth_foundational::FoundationalPerformanceLayoutIntent::Custom,
    ];
    layouts.sort();
    assert_eq!(
        layouts,
        vec![
            worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
            worth_foundational::FoundationalPerformanceLayoutIntent::Packed,
            worth_foundational::FoundationalPerformanceLayoutIntent::Custom,
        ]
    );

    let mut temperatures = vec![
        worth_foundational::FoundationalPerformanceExecutionTemperature::SupportOnly,
        worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
        worth_foundational::FoundationalPerformanceExecutionTemperature::ColdPath,
    ];
    temperatures.sort();
    assert_eq!(
        temperatures,
        vec![
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
            worth_foundational::FoundationalPerformanceExecutionTemperature::ColdPath,
            worth_foundational::FoundationalPerformanceExecutionTemperature::SupportOnly,
        ]
    );

    let front_door = worth_foundational::performance_api::common_path::performance();
    assert_eq!(
        front_door.execution_temperature_definitions(),
        foundational_performance_execution_temperature_definitions()
    );
}

#[test]
fn primitive_legality_floor_rejects_obvious_hot_path_and_debt_contradictions() {
    let hot_path = [worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation];
    let hot_excluded = [
        worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction,
        worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
    ];

    assert_eq!(
        evaluate_performance_primitive_legality(
            worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution,
            worth_foundational::FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim,
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            worth_foundational::FoundationalPerformanceFallbackDebtPosture::Debt,
            &hot_path,
            &hot_excluded,
        ),
        Err(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::SupportDerivedClaimsCannotUseHotPathTemperature
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution,
            worth_foundational::FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
            worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath,
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified,
            &hot_path,
            &[],
        ),
        Err(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::DebtOrDeferredEvidenceCannotClaimVerifiedPosture
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            worth_foundational::FoundationalPerformanceBoundary::BoundaryMaterialization,
            worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission,
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred,
            &hot_path,
            &hot_excluded,
        ),
        Err(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::BoundaryMaterializationCannotClaimHotPathTemperature
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution,
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::StaleSupport,
            worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified,
            &hot_path,
            &hot_excluded,
        ),
        Err(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::HotPathClaimsCannotUseReplayOrStaleFreshness
        )
    );

    assert_eq!(
        evaluate_performance_primitive_legality(
            worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution,
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
            worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified,
            &[
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
                worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction,
            ],
            &hot_excluded,
        ),
        Err(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::HotPathIncludedWorkMustExcludeReplaySupportAndForensics
        )
    );
}

#[test]
fn front_door_keeps_legality_and_definition_surfaces_in_the_common_path() {
    let front_door = worth_foundational::performance_api::common_path::performance();
    let result = front_door.evaluate_primitive_legality(
        worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution,
        worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
        worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified,
        &[worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation],
        &[
            worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction,
            worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
        ],
    );

    assert_eq!(result, Ok(()));
}
