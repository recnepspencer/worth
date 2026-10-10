use super::*;

struct Feature;
struct Computation;
struct Artifact;
struct OtherFeature;

fn expectation() -> ManagedComputationExpectation {
    ManagedComputationExpectation {
        feature_identity: "feature",
        identity: "worth.query.tests.required-computation.v1",
        feature_type: TypeId::of::<Feature>(),
        computation_type: TypeId::of::<Computation>(),
        output_artifact_type: TypeId::of::<Artifact>(),
    }
}

#[test]
fn equal_computation_names_are_denied_in_declared_feature_order() {
    let alpha = ManagedComputationExpectation {
        feature_identity: "alpha-feature",
        identity: "shared-computation",
        ..expectation()
    };
    let zulu = ManagedComputationExpectation {
        feature_identity: "zulu-feature",
        identity: "shared-computation",
        feature_type: TypeId::of::<OtherFeature>(),
        computation_type: TypeId::of::<OtherFeature>(),
        ..expectation()
    };
    let mismatched_alpha = HashMap::from([(
        TypeId::of::<Computation>(),
        InstalledManagedComputationOwner {
            feature_type: TypeId::of::<OtherFeature>(),
            computation_type: TypeId::of::<Computation>(),
            output_artifact_type: TypeId::of::<Artifact>(),
        },
    )]);
    for entries in [
        [
            (alpha.computation_type, alpha),
            (zulu.computation_type, zulu),
        ],
        [
            (zulu.computation_type, zulu),
            (alpha.computation_type, alpha),
        ],
    ] {
        let denied =
            validate_managed_computation_inventory(&mismatched_alpha, &entries).unwrap_err();
        assert_eq!(
            denied.kind(),
            DenialKind::ManagedComputationOwnerMeaningMismatch
        );
        assert_eq!(denied.subject(), "shared-computation");
    }
}

#[test]
fn declared_managed_computation_requires_its_exact_owner_inventory() {
    let declared = Vec::from([(TypeId::of::<Computation>(), expectation())]);
    let missing = validate_managed_computation_inventory(&HashMap::new(), &declared)
        .expect_err("a declared computation without its owner must be denied");
    assert_eq!(missing.kind(), DenialKind::MissingManagedComputationOwner);

    let mismatched = HashMap::from([(
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

    let exact = HashMap::from([(
        TypeId::of::<Computation>(),
        InstalledManagedComputationOwner {
            feature_type: TypeId::of::<Feature>(),
            computation_type: TypeId::of::<Computation>(),
            output_artifact_type: TypeId::of::<Artifact>(),
        },
    )]);
    validate_managed_computation_inventory(&exact, &declared)
        .expect("the exact declared owner inventory validates");

    let foreign = HashMap::from([
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
