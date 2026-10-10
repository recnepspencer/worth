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
    let declared = HashMap::from([(TypeId::of::<Computation>(), expectation())]);
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

#[test]
fn missing_owners_are_named_in_declared_computation_order() {
    let a = TypeId::of::<Computation>();
    let b = TypeId::of::<OtherFeature>();
    let (alpha_type, zulu_type) = if a > b { (a, b) } else { (b, a) };
    let alpha = ManagedComputationExpectation {
        identity: "alpha-computation",
        computation_type: alpha_type,
        ..expectation()
    };
    let zulu = ManagedComputationExpectation {
        identity: "zulu-computation",
        computation_type: zulu_type,
        ..expectation()
    };
    for reverse in [false, true] {
        let mut entries = [(alpha_type, alpha), (zulu_type, zulu)];
        if reverse {
            entries.reverse();
        }
        let denied =
            validate_managed_computation_inventory(&HashMap::new(), &HashMap::from(entries))
                .unwrap_err();
        assert_eq!(denied.subject(), "alpha-computation");
    }
}
