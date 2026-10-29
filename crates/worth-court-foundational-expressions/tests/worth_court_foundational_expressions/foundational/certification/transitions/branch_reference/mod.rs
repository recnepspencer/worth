#[allow(unused_imports)] use crate::certification::ordinary_context::transitions::branch_reference::*;
use {serde::Deserialize, serde::Serialize};
use {worth_foundational::FoundationalBranchComparisonBasis, worth_foundational::FoundationalBranchForkBasis, worth_foundational::FoundationalBranchId, worth_foundational::FoundationalBranchReferenceGeneration, worth_foundational::FoundationalBranchReferenceGenerationAdvanceDenial, worth_foundational::FoundationalBranchReferenceMismatchAxis, worth_foundational::FoundationalBranchReferenceMovement, worth_foundational::FoundationalBranchReferenceMovementKind, worth_foundational::FoundationalBranchReferenceObservation, worth_foundational::FoundationalBranchTarget, worth_foundational::FoundationalBranchTargetBasis, worth_foundational::FoundationalBranchTargetEncoding, worth_foundational::FoundationalBranchTargetEncodingConstructionDenial};

#[test]
fn target_keeps_empty_and_basis_as_explicit_distinct_variants() {
    let empty: FoundationalBranchTarget<RelationalTarget> = worth_foundational::FoundationalBranchTarget::empty();
    let basis = worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9));

    assert!(empty.is_empty());
    assert!(empty.as_basis().is_none());
    assert!(!basis.is_empty());
    assert_eq!(basis.as_basis().expect("basis is present").commit, 4);
    assert_ne!(empty, basis);
}

#[test]
fn reference_observation_equality_is_structural_and_owner_affine() {
    let expected = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9)),
        2,
    );
    let independently_constructed = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9)),
        2,
    );
    assert_eq!(expected, independently_constructed);

    let wrong_branch = relational_observation(
        "maintenance",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9)),
        2,
    );
    let wrong_runtime = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-b", 4, 9)),
        2,
    );
    let wrong_generation = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9)),
        3,
    );

    assert_ne!(expected, wrong_branch);
    assert_ne!(expected, wrong_runtime);
    assert_ne!(expected, wrong_generation);
    assert_ne!(
        expected.canonical_encoding(),
        wrong_branch.canonical_encoding()
    );
    assert_ne!(
        expected.canonical_encoding(),
        wrong_runtime.canonical_encoding()
    );
    assert_ne!(
        expected.canonical_encoding(),
        wrong_generation.canonical_encoding()
    );
}

#[test]
fn fork_comparison_and_movement_retain_complete_observations() {
    let source = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9)),
        2,
    );
    let target = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 5, 12)),
        3,
    );
    let fork = worth_foundational::FoundationalBranchForkBasis::new(source.clone());
    let comparison = worth_foundational::FoundationalBranchComparisonBasis::new(source.clone());
    let movement = worth_foundational::FoundationalBranchReferenceMovement::new(
        source.clone(),
        target.clone(),
        worth_foundational::FoundationalBranchReferenceMovementKind::Truth,
    );

    assert_eq!(fork.source_observation(), &source);
    assert!(comparison.compare(&source).is_ok());
    assert!(comparison.compare(&target).is_err());
    assert_eq!(movement.before(), &source);
    assert_eq!(movement.after(), &target);
    assert_eq!(
        movement.kind(),
        worth_foundational::FoundationalBranchReferenceMovementKind::Truth
    );
}

#[test]
fn mismatch_reports_all_structural_axes_in_deterministic_order() {
    let expected = relational_observation(
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(relational_target("runtime-a", 4, 9)),
        2,
    );
    let observed = relational_observation("maintenance", worth_foundational::FoundationalBranchTarget::Empty, 3);

    let mismatch = expected.compare(&observed).expect_err("three axes differ");
    assert_eq!(
        mismatch.axes(),
        &[
            worth_foundational::FoundationalBranchReferenceMismatchAxis::BranchIdentity,
            worth_foundational::FoundationalBranchReferenceMismatchAxis::TargetBasis,
            worth_foundational::FoundationalBranchReferenceMismatchAxis::ReferenceGeneration,
        ]
    );
    assert_eq!(mismatch.expected(), &expected);
    assert_eq!(mismatch.observed(), &observed);
}

#[test]
fn generation_advancement_is_checked_and_never_wraps() {
    let initial = worth_foundational::FoundationalBranchReferenceGeneration::initial();
    assert_eq!(initial.get(), 0);
    assert_eq!(
        initial.checked_advance().expect("zero advances"),
        worth_foundational::FoundationalBranchReferenceGeneration::new(1)
    );

    let maximum = worth_foundational::FoundationalBranchReferenceGeneration::new(u64::MAX);
    assert_eq!(
        maximum.checked_advance(),
        Err(worth_foundational::FoundationalBranchReferenceGenerationAdvanceDenial::Overflow)
    );
}

#[test]
fn canonical_encoding_is_versioned_variant_tagged_and_round_trips() {
    let empty = relational_observation("storm", worth_foundational::FoundationalBranchTarget::empty(), 3);
    assert_eq!(
        empty.canonical_encoding(),
        hex_bytes("574f5254482d4252414e43482d5245464552454e43450001000000000000000573746f726d000000000000000003")
    );

    let signal = worth_foundational::FoundationalBranchReferenceObservation::new(
        branch("signal"),
        worth_foundational::FoundationalBranchTarget::basis(SignalTarget {
            graph: "graph-a".to_owned(),
            definition: 7,
            snapshot: 11,
        }),
        worth_foundational::FoundationalBranchReferenceGeneration::new(4),
    );
    let encoded = serde_json::to_vec(&signal).expect("descriptive reference serializes");
    let decoded: FoundationalBranchReferenceObservation<SignalTarget> =
        serde_json::from_slice(&encoded).expect("descriptive reference deserializes");
    assert_eq!(decoded, signal);
    assert_eq!(
        signal.canonical_encoding(),
        hex_bytes(
            "574f5254482d4252414e43482d5245464552454e4345000100000000000000067369676e616c010000000000000020776f7274682e7369676e616c2e646566696e6974696f6e2d736e617073686f740001000000000000000c67726170682d613a373a31310000000000000004"
        )
    );
}

#[test]
fn malformed_descriptive_transport_cannot_bypass_structural_validation() {
    let empty_branch = serde_json::from_str::<FoundationalBranchId>("\"\"")
        .expect_err("empty branch ids remain invalid after deserialization");
    assert!(empty_branch.to_string().contains("invalid branch id"));

    let whitespace_branch = serde_json::from_str::<FoundationalBranchId>("\"   \"")
        .expect_err("whitespace-only branch ids remain invalid after deserialization");
    assert!(whitespace_branch.to_string().contains("invalid branch id"));

    let nested_whitespace_branch = serde_json::from_value::<
        FoundationalBranchReferenceObservation<SignalTarget>,
    >(serde_json::json!({
        "branch_id": "   ",
        "target": "Empty",
        "generation": 0
    }))
    .expect_err("nested observations must validate branch identity during decode");
    assert!(nested_whitespace_branch
        .to_string()
        .contains("invalid branch id"));

    let empty_domain = serde_json::from_str::<FoundationalBranchTargetEncoding>(
        r#"{"domain":"","schema_version":1,"bytes":[]}"#,
    )
    .expect_err("empty target domains remain invalid after deserialization");
    assert!(empty_domain.to_string().contains("invalid target encoding"));

    let whitespace_domain = serde_json::from_str::<FoundationalBranchTargetEncoding>(
        r#"{"domain":" ","schema_version":1,"bytes":[]}"#,
    )
    .expect_err("whitespace-only target domains remain invalid after deserialization");
    assert!(whitespace_domain
        .to_string()
        .contains("invalid target encoding"));

    let zero_version = serde_json::from_str::<FoundationalBranchTargetEncoding>(
        r#"{"domain":"test","schema_version":0,"bytes":[]}"#,
    )
    .expect_err("zero target schema versions remain invalid after deserialization");
    assert!(zero_version.to_string().contains("invalid target encoding"));

    assert_eq!(
        worth_foundational::FoundationalBranchTargetEncoding::new("test", 0, Vec::new()),
        Err(worth_foundational::FoundationalBranchTargetEncodingConstructionDenial::ZeroSchemaVersion)
    );
}
