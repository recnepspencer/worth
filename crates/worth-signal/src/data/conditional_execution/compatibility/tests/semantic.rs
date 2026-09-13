use super::super::super::{
    SignalConditionalArtifactReuse, SignalConditionalCondition, SignalConditionalVersionComparator,
};
use super::super::SignalConditionalComparatorPosition;
use crate::data::graph::SignalGraph;

use super::support::{
    absolute, base_definition, claim_owner, exclusive, float64, inclusive, install_at,
    install_fresh, integer, mask, mismatch_kind, portable_definition, relative, threshold,
    with_artifact_reuse, with_condition, with_dependency_aspects, with_dependency_comparator,
    with_output_comparator, with_threshold, with_trigger_aspects, SemanticMismatchKind,
};

#[test]
fn every_threshold_and_contract_field_reports_its_first_semantic_drift() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = claim_owner(&mut graph);
    let current = install_at(&mut graph, &owner, node, base_definition());
    let cases = [
        (
            "condition class",
            with_condition(SignalConditionalCondition::RuntimePredicate),
            SemanticMismatchKind::ConditionClass,
        ),
        (
            "threshold value",
            with_threshold(threshold(
                11,
                "millimeter",
                integer(),
                absolute(),
                inclusive(),
            )),
            SemanticMismatchKind::ThresholdValue,
        ),
        (
            "threshold unit",
            with_threshold(threshold(
                10,
                "centimeter",
                integer(),
                absolute(),
                inclusive(),
            )),
            SemanticMismatchKind::ThresholdUnitIdentity,
        ),
        (
            "threshold value family",
            with_threshold(threshold(
                10,
                "millimeter",
                float64(),
                absolute(),
                inclusive(),
            )),
            SemanticMismatchKind::ThresholdValueFamily,
        ),
        (
            "threshold comparison domain",
            with_threshold(threshold(
                10,
                "millimeter",
                integer(),
                relative(),
                inclusive(),
            )),
            SemanticMismatchKind::ThresholdComparisonDomain,
        ),
        (
            "threshold boundary",
            with_threshold(threshold(
                10,
                "millimeter",
                integer(),
                absolute(),
                exclusive(),
            )),
            SemanticMismatchKind::ThresholdBoundary,
        ),
        (
            "dependency aspects",
            with_dependency_aspects(mask(4)),
            SemanticMismatchKind::DependencyAspects,
        ),
        (
            "trigger aspects",
            with_trigger_aspects(mask(5)),
            SemanticMismatchKind::TriggerAspects,
        ),
        (
            "dependency comparator class",
            with_dependency_comparator(SignalConditionalVersionComparator::OutputIdentity),
            SemanticMismatchKind::ComparatorClass(SignalConditionalComparatorPosition::Dependency),
        ),
        (
            "output comparator class",
            with_output_comparator(SignalConditionalVersionComparator::OutputIdentity),
            SemanticMismatchKind::ComparatorClass(SignalConditionalComparatorPosition::Output),
        ),
        (
            "artifact reuse",
            with_artifact_reuse(SignalConditionalArtifactReuse::OutputEquivalent),
            SemanticMismatchKind::ArtifactReuseClass,
        ),
    ];

    for (name, candidate_definition, expected) in cases {
        let candidate = install_at(&mut graph, &owner, node, candidate_definition);
        let mismatch = current
            .compare_semantic_continuity(&candidate)
            .err()
            .unwrap_or_else(|| panic!("{name} drift unexpectedly admitted"));
        assert_eq!(mismatch_kind(mismatch.mismatch()), expected, "{name}");
    }
}

#[test]
fn aspect_filter_mask_is_compared_as_condition_meaning() {
    let current = install_fresh(with_condition(SignalConditionalCondition::AspectFilter(
        mask(6),
    )));
    let candidate = install_fresh(with_condition(SignalConditionalCondition::AspectFilter(
        mask(7),
    )));

    let mismatch = current.compare_semantic_continuity(&candidate).unwrap_err();
    assert_eq!(
        mismatch_kind(mismatch.mismatch()),
        SemanticMismatchKind::AspectFilterMask
    );
}

#[test]
fn comparator_tolerance_is_compared_for_each_semantic_position() {
    let mut dependency_current = portable_definition();
    dependency_current.dependency_comparator = SignalConditionalVersionComparator::Tolerance(3);
    let mut dependency_candidate = dependency_current.clone();
    dependency_candidate.dependency_comparator = SignalConditionalVersionComparator::Tolerance(4);
    assert_eq!(
        mismatch_kind(
            install_fresh(dependency_current)
                .compare_semantic_continuity(&install_fresh(dependency_candidate))
                .unwrap_err()
                .mismatch()
        ),
        SemanticMismatchKind::ComparatorTolerance(SignalConditionalComparatorPosition::Dependency)
    );

    let mut output_current = portable_definition();
    output_current.output_comparator = SignalConditionalVersionComparator::Tolerance(3);
    let mut output_candidate = output_current.clone();
    output_candidate.output_comparator = SignalConditionalVersionComparator::Tolerance(4);
    assert_eq!(
        mismatch_kind(
            install_fresh(output_current)
                .compare_semantic_continuity(&install_fresh(output_candidate))
                .unwrap_err()
                .mismatch()
        ),
        SemanticMismatchKind::ComparatorTolerance(SignalConditionalComparatorPosition::Output)
    );
}

#[test]
fn owner_known_installed_roles_preserve_semantics_across_reinstallation() {
    let condition_current =
        install_fresh(with_condition(SignalConditionalCondition::RuntimePredicate));
    let condition_candidate =
        install_fresh(with_condition(SignalConditionalCondition::RuntimePredicate));
    let _condition_continuity = condition_current
        .compare_semantic_continuity(&condition_candidate)
        .expect("Signal owns the installed predicate role across graph generations");

    let mut comparator_definition = portable_definition();
    comparator_definition.dependency_comparator =
        SignalConditionalVersionComparator::RuntimeResolved;
    let comparator_current = install_fresh(comparator_definition.clone());
    let comparator_candidate = install_fresh(comparator_definition);
    let _comparator_continuity = comparator_current
        .compare_semantic_continuity(&comparator_candidate)
        .expect("Signal owns the installed dependency-comparator role");

    let mut reuse_definition = portable_definition();
    reuse_definition.artifact_reuse = SignalConditionalArtifactReuse::RuntimeResolved;
    let reuse_current = install_fresh(reuse_definition.clone());
    let reuse_candidate = install_fresh(reuse_definition);
    let _reuse_continuity = reuse_current
        .compare_semantic_continuity(&reuse_candidate)
        .expect("Signal owns the installed artifact-reuse role");
}

#[test]
fn semantic_comparison_reports_exact_owner_work_on_success_and_denial() {
    let current = install_fresh(base_definition());
    let candidate = install_fresh(base_definition());
    let continuity = current.compare_semantic_continuity(&candidate).unwrap();
    assert_eq!(continuity.work().semantic_dimensions_inspected(), 12);
    assert_eq!(continuity.work().affinity_dimensions_inspected(), 0);

    let drifted = install_fresh(with_condition(SignalConditionalCondition::TemporalWake));
    let denial = current.compare_semantic_continuity(&drifted).unwrap_err();
    assert_eq!(denial.work().semantic_dimensions_inspected(), 1);
    assert_eq!(denial.work().affinity_dimensions_inspected(), 0);
}
