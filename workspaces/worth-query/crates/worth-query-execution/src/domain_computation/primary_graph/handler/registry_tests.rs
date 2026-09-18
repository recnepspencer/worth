use super::*;

struct Feature;
struct Computation;
struct Artifact;
struct OtherFeature;

fn expectation() -> ManagedComputationExpectation {
    ManagedComputationExpectation {
        identity: "worth.query.tests.required-computation.v1",
        feature_type: TypeId::of::<Feature>(),
        computation_type: TypeId::of::<Computation>(),
        output_artifact_type: TypeId::of::<Artifact>(),
    }
}

#[test]
fn declared_managed_computation_requires_its_exact_owner_inventory() {
    let declared = BTreeMap::from([(TypeId::of::<Computation>(), expectation())]);
    let missing = validate_managed_computation_inventory(&BTreeMap::new(), &declared)
        .expect_err("a declared computation without its owner must be denied");
    assert_eq!(missing.kind(), DenialKind::MissingManagedComputationOwner);

    let mismatched = BTreeMap::from([(
        TypeId::of::<Computation>(),
        InstalledManagedComputationOwner {
            feature_type: TypeId::of::<OtherFeature>(),
            computation_type: TypeId::of::<Computation>(),
            output_artifact_type: TypeId::of::<Artifact>(),
        },
    )]);
    let mismatch = validate_managed_computation_inventory(&mismatched, &declared)
        .expect_err("an owner bound to different feature meaning must be denied");
    assert_eq!(
        mismatch.kind(),
        DenialKind::ManagedComputationOwnerMeaningMismatch
    );

    let exact = BTreeMap::from([(
        TypeId::of::<Computation>(),
        InstalledManagedComputationOwner {
            feature_type: TypeId::of::<Feature>(),
            computation_type: TypeId::of::<Computation>(),
            output_artifact_type: TypeId::of::<Artifact>(),
        },
    )]);
    validate_managed_computation_inventory(&exact, &declared)
        .expect("the exact declared owner inventory validates");

    let foreign = BTreeMap::from([
        (
            TypeId::of::<Computation>(),
            exact[&TypeId::of::<Computation>()],
        ),
        (
            TypeId::of::<OtherFeature>(),
            InstalledManagedComputationOwner {
                feature_type: TypeId::of::<OtherFeature>(),
                computation_type: TypeId::of::<OtherFeature>(),
                output_artifact_type: TypeId::of::<Artifact>(),
            },
        ),
    ]);
    let denial = validate_managed_computation_inventory(&foreign, &declared)
        .expect_err("an undeclared owner must be denied");
    assert_eq!(denial.kind(), DenialKind::ForeignManagedComputationOwner);
}
