pub(crate) use worth_query_testkit_query::facade::query::suite::conditional_node_canonicality::*;

#[test]
fn declaration_order_converges_and_every_condition_family_is_portable() {
    let nodes = representative_nodes();
    let direct = definition(nodes.clone()).into_portable();
    let reversed = definition(nodes.into_iter().rev().collect()).into_portable();

    assert_eq!(direct, reversed);
    assert_eq!(direct.semantics().conditional_nodes.len(), 5);
    assert!(direct
        .semantics()
        .conditional_nodes
        .iter()
        .all(|node| !node.identity().is_empty()));
}

#[test]
fn domain_condition_parameters_are_named_canonical_and_not_positional() {
    let left =
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([
            domain::WorthQueryPortableConditionParameter::u64("maximum", 9).unwrap(),
            domain::WorthQueryPortableConditionParameter::u64("minimum", 2).unwrap(),
        ])
        .unwrap();
    let right =
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([
            domain::WorthQueryPortableConditionParameter::u64("minimum", 2).unwrap(),
            domain::WorthQueryPortableConditionParameter::u64("maximum", 9).unwrap(),
        ])
        .unwrap();
    assert_eq!(
        canonical_identity(vec![conditional_node_result(
            "named-parameters",
            dependency(domain::WorthQuerySemanticLocality::SourceRecord),
            left,
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        )
        .unwrap()]),
        canonical_identity(vec![conditional_node_result(
            "named-parameters",
            dependency(domain::WorthQuerySemanticLocality::SourceRecord),
            right,
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        )
        .unwrap()])
    );
    assert!(
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([
            domain::WorthQueryPortableConditionParameter::u64("limit", 1).unwrap(),
            domain::WorthQueryPortableConditionParameter::u64("limit", 2).unwrap(),
        ])
        .is_err()
    );
}

#[test]
fn every_conditional_posture_dimension_changes_canonical_operation_meaning() {
    assert_drift(
        authored_node(
            domain::WorthQueryConditionalEvaluationCondition::always_eligible(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
    );
    assert_drift(
        on_demand_node::<ManualRefresh>(),
        on_demand_node::<AlternateRefresh>(),
    );
    assert_drift(temporal_node(1_000_000), temporal_node(2_000_000));

    let base = || {
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        )
    };
    assert_drift(
        base(),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::registered::<GeometryTolerance>(),
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
    );
    assert_drift(
        base(),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
    );
    assert_drift(
        base(),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::OutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
    );
    assert_drift(
        base(),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::EagerOnEligibleInvalidation,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
    );
    assert_drift(
        base(),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::Durable,
            domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
        ),
    );
    assert_drift(
        base(),
        authored_node(
            aspect_condition(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
            domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            domain::WorthQueryOutputRelationship::IsOperationOutput,
        ),
    );
}

#[test]
fn dependency_basis_preserves_foundational_and_relational_meaning() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let basis = dependency.canonical_basis().unwrap();
    assert_eq!(basis.foundational().payload().sequences().len(), 3);
    assert_eq!(basis.relational_binding(), "entity-field:id");
    assert_eq!(
        basis.relevant_changes(),
        [RelationalAspectChangeKind::FieldSet]
    );
    assert_eq!(
        dependency.contract().identity(),
        AspectIdentity(0x9140_0001)
    );
    assert_eq!(dependency.projection_mask().paths().len(), 1);
}

#[test]
fn thresholds_reject_nonfinite_negative_unit_and_value_family_drift() {
    for value in [f64::NAN, f64::INFINITY, -0.01] {
        assert_eq!(
            domain::WorthQueryDeltaThreshold::new::<Millimeters>(
                AspectValue::Float64(CanonicalF64::from_f64(value)),
                domain::WorthQueryDeltaComparisonDomain::AbsoluteDifference,
                domain::WorthQueryThresholdBoundary::Inclusive,
            ),
            Err("invalid-delta-threshold")
        );
    }
    assert_ne!(
        threshold::<Millimeters>().unit(),
        threshold::<Seconds>().unit()
    );
    assert_eq!(
        domain::WorthQueryDeltaThreshold::new::<Millimeters>(
            AspectValue::UInt64(1),
            domain::WorthQueryDeltaComparisonDomain::AbsoluteDifference,
            domain::WorthQueryThresholdBoundary::Inclusive,
        ),
        Err("delta-threshold-value-family-mismatch")
    );
    assert_eq!(
        domain::WorthQueryDeltaThreshold::new::<InvalidUnitIdentity>(
            AspectValue::Float64(CanonicalF64::from_f64(1.0)),
            domain::WorthQueryDeltaComparisonDomain::AbsoluteDifference,
            domain::WorthQueryThresholdBoundary::Inclusive,
        ),
        Err("invalid-portable-quantity-unit-identity")
    );
    let textual_dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    assert_eq!(
        conditional_node_result(
            "textual-threshold",
            textual_dependency.clone(),
            domain::WorthQueryConditionalEvaluationCondition::delta_threshold(
                textual_dependency,
                threshold::<Millimeters>(),
            ),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        )
        .unwrap_err(),
        "delta-threshold-requires-one-numeric-scalar"
    );
}

#[test]
fn invalid_binding_trigger_and_cadence_deny_before_installation() {
    let denial = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("model").unwrap(),
        identity_contract(),
        identity_mask(),
        AspectBinding::RelationSourceEndpoint,
        domain::WorthQuerySemanticLocality::SourceRecord,
        [RelationalAspectChangeKind::FieldSet],
    )
    .unwrap_err();
    assert_eq!(
        denial,
        domain::WorthQuerySemanticTruthDependencyDenial::ChangeMeaningDoesNotMatchBinding
    );
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    assert_eq!(
        conditional_node_result(
            "on-demand-mismatch",
            dependency.clone(),
            domain::WorthQueryConditionalEvaluationCondition::on_demand(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        )
        .unwrap_err(),
        "on-demand-condition-trigger-mismatch"
    );
    assert_eq!(
        conditional_node_result(
            "zero-cadence",
            dependency,
            domain::WorthQueryConditionalEvaluationCondition::temporal(
                domain::WorthQueryTemporalCondition::IntervalNanoseconds(0),
            ),
            domain::WorthQueryConditionalTrigger::Temporal(
                domain::WorthQueryTemporalWake::MonotonicClock,
            ),
            domain::WorthQueryMaintenancePosture::Temporal,
        )
        .unwrap_err(),
        "invalid-temporal-condition-duration"
    );
}
