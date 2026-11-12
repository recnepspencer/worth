use super::*;
use worth_foundational::facade::AspectValue;
use worth_signal::facade::{
    SignalThresholdBoundary, SignalThresholdComparisonDomain, SignalThresholdValueFamily,
};

#[test]
fn conditional_definition_retention_includes_the_threshold_owners_unit_storage() {
    let mut units = String::with_capacity(8192);
    units.push_str("integer units");
    let expected = units.capacity() as u64;
    let threshold = SignalDeltaThresholdContract::new(
        AspectValue::UInt64(7),
        units,
        SignalThresholdValueFamily::Integer,
        SignalThresholdComparisonDomain::AbsoluteDifference,
        SignalThresholdBoundary::Inclusive,
    );
    let make = |condition| {
        BridgeConditionalContract::new(BridgeConditionalContractParts {
            identity: Arc::from("threshold-retention"),
            dependency_count: 1,
            condition_dependency_ordinals: vec![0],
            condition,
            dependency_comparator: SignalConditionalVersionComparator::Exact,
            output_comparator: SignalConditionalVersionComparator::Exact,
            artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
        })
    };
    let baseline = make(BridgeConditionalCondition::Always)
        .retained_heap_bytes()
        .unwrap();
    let authored = make(BridgeConditionalCondition::DeltaThreshold(threshold));
    assert_eq!(authored.retained_heap_bytes().unwrap() - baseline, expected);
}
