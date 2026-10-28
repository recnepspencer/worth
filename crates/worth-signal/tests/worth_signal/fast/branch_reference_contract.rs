use worth_foundational::{
    FoundationalBranchReferenceGeneration, FoundationalBranchReferenceMismatchAxis,
    FoundationalBranchTarget, FoundationalBranchTargetBasis,
};
use worth_signal::facade::branch::{signal_branch_observation, SignalBranchTarget};

#[test]
fn signal_basis_identity_lowers_to_the_shared_target_descriptor() {
    let target =
        SignalBranchTarget::new("graph-a", 3, Some(11), None).expect("valid graph identity");
    assert_eq!(target.snapshot_id(), Some(11));
    assert_eq!(target.definition_basis(), 3);

    let observation = signal_branch_observation(
        "graph-a",
        7,
        "storm",
        FoundationalBranchTarget::basis(target.clone()),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("owner branch lowers to the shared grammar");
    assert_eq!(
        observation.branch_id().as_str(),
        "signal/7:graph-a/1:7/5:storm"
    );
    assert_eq!(
        target.canonical_encoding().bytes(),
        hex_bytes("000000000000000767726170682d61000000000000000301000000000000000b00")
    );

    let foreign_observation = signal_branch_observation(
        "graph-b",
        7,
        "storm",
        FoundationalBranchTarget::basis(
            SignalBranchTarget::new(
                "graph-b",
                target.definition_basis(),
                target.snapshot_id(),
                target.restore_snapshot_id(),
            )
            .expect("valid graph identity"),
        ),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("foreign graph lowers to the shared grammar");
    let mismatch = observation
        .compare(&foreign_observation)
        .expect_err("foreign graph twins must not compare equal");
    assert_eq!(
        mismatch.axes(),
        &[
            FoundationalBranchReferenceMismatchAxis::BranchIdentity,
            FoundationalBranchReferenceMismatchAxis::TargetBasis,
        ]
    );
}

#[test]
fn signal_target_canonical_encoding_includes_graph_identity() {
    let first =
        SignalBranchTarget::new("graph-a", 3, Some(11), None).expect("valid graph identity");
    let second =
        SignalBranchTarget::new("graph-b", 3, Some(11), None).expect("valid graph identity");
    assert_ne!(first.canonical_encoding(), second.canonical_encoding());

    let definition_variant =
        SignalBranchTarget::new("graph-a", 4, Some(11), None).expect("valid graph identity");
    let snapshot_variant =
        SignalBranchTarget::new("graph-a", 3, Some(12), None).expect("valid graph identity");
    let restore_variant =
        SignalBranchTarget::new("graph-a", 3, Some(11), Some(13)).expect("valid graph identity");
    assert_ne!(
        first.canonical_encoding(),
        definition_variant.canonical_encoding()
    );
    assert_ne!(
        first.canonical_encoding(),
        snapshot_variant.canonical_encoding()
    );
    assert_ne!(
        first.canonical_encoding(),
        restore_variant.canonical_encoding()
    );
}

#[test]
fn malformed_signal_target_transport_cannot_admit_blank_graph_identity() {
    let denial = serde_json::from_str::<SignalBranchTarget>(
        r#"{"graph_instance_id":"   ","definition_basis":1,"snapshot_id":null,"restore_snapshot_id":null}"#,
    )
    .expect_err("blank graph identities must be rejected during transport decode");
    assert!(denial.to_string().contains("invalid Signal branch target"));
}

#[test]
fn signal_owner_branch_identity_is_injective_and_validated() {
    let first = signal_branch_observation(
        "graph/a",
        7,
        "b",
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    let second = signal_branch_observation(
        "graph",
        7,
        "a/b",
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    assert_ne!(first.branch_id(), second.branch_id());
    let branch_seven = signal_branch_observation(
        "graph",
        7,
        "storm",
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    let branch_eight = signal_branch_observation(
        "graph",
        8,
        "storm",
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    assert_ne!(branch_seven.branch_id(), branch_eight.branch_id());
    assert!(signal_branch_observation(
        "   ",
        7,
        "storm",
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .is_err());
    assert!(signal_branch_observation(
        "graph",
        7,
        "   ",
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .is_err());
}

#[test]
fn signal_observation_rejects_foreign_target_graph() {
    let target =
        SignalBranchTarget::new("graph-b", 3, Some(11), None).expect("valid graph identity");
    let denial = signal_branch_observation(
        "graph-a",
        7,
        "storm",
        FoundationalBranchTarget::basis(target),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect_err("foreign graph target must not cross the observation adapter");
    assert!(matches!(
        denial,
        worth_signal::facade::branch::SignalBranchObservationConstructionDenial::GraphInstanceMismatch {
            ..
        }
    ));
}

fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("hex is utf8"), 16)
                .expect("valid hex")
        })
        .collect()
}
