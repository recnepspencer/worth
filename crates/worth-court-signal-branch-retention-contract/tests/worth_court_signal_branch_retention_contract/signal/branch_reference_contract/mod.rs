#[allow(unused_imports)]
use crate::branch_reference_contract::ordinary_context::*;
use worth_foundational::FoundationalBranchTargetBasis;
use {
    worth_signal::facade::branch::signal_branch_observation,
    worth_signal::facade::branch::SignalBranchTarget,
};

#[test]
fn signal_basis_identity_lowers_to_the_shared_target_descriptor() {
    let target =
        worth_signal::facade::branch::SignalBranchTarget::new("graph-a", 3, Some(11), None)
            .expect("valid graph identity");
    assert_eq!(target.snapshot_id(), Some(11));
    assert_eq!(target.definition_basis(), 3);

    let observation = signal_branch_observation(
        "graph-a",
        7,
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(target.clone()),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
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
        worth_foundational::FoundationalBranchTarget::basis(
            worth_signal::facade::branch::SignalBranchTarget::new(
                "graph-b",
                target.definition_basis(),
                target.snapshot_id(),
                target.restore_snapshot_id(),
            )
            .expect("valid graph identity"),
        ),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("foreign graph lowers to the shared grammar");
    let mismatch = observation
        .compare(&foreign_observation)
        .expect_err("foreign graph twins must not compare equal");
    assert_eq!(
        mismatch.axes(),
        &[
            worth_foundational::FoundationalBranchReferenceMismatchAxis::BranchIdentity,
            worth_foundational::FoundationalBranchReferenceMismatchAxis::TargetBasis,
        ]
    );
}

#[test]
fn signal_target_canonical_encoding_includes_graph_identity() {
    let first = worth_signal::facade::branch::SignalBranchTarget::new("graph-a", 3, Some(11), None)
        .expect("valid graph identity");
    let second =
        worth_signal::facade::branch::SignalBranchTarget::new("graph-b", 3, Some(11), None)
            .expect("valid graph identity");
    assert_ne!(first.canonical_encoding(), second.canonical_encoding());

    let definition_variant =
        worth_signal::facade::branch::SignalBranchTarget::new("graph-a", 4, Some(11), None)
            .expect("valid graph identity");
    let snapshot_variant =
        worth_signal::facade::branch::SignalBranchTarget::new("graph-a", 3, Some(12), None)
            .expect("valid graph identity");
    let restore_variant =
        worth_signal::facade::branch::SignalBranchTarget::new("graph-a", 3, Some(11), Some(13))
            .expect("valid graph identity");
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
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    let second = signal_branch_observation(
        "graph",
        7,
        "a/b",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    assert_ne!(first.branch_id(), second.branch_id());
    let branch_seven = signal_branch_observation(
        "graph",
        7,
        "storm",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    let branch_eight = signal_branch_observation(
        "graph",
        8,
        "storm",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid owner components");
    assert_ne!(branch_seven.branch_id(), branch_eight.branch_id());
    assert!(signal_branch_observation(
        "   ",
        7,
        "storm",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .is_err());
    assert!(signal_branch_observation(
        "graph",
        7,
        "   ",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .is_err());
}

#[test]
fn signal_observation_rejects_foreign_target_graph() {
    let target =
        worth_signal::facade::branch::SignalBranchTarget::new("graph-b", 3, Some(11), None)
            .expect("valid graph identity");
    let denial = signal_branch_observation(
        "graph-a",
        7,
        "storm",
        worth_foundational::FoundationalBranchTarget::basis(target),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect_err("foreign graph target must not cross the observation adapter");
    assert!(matches!(
        denial,
        worth_signal::facade::branch::SignalBranchObservationConstructionDenial::GraphInstanceMismatch {
            ..
        }
    ));
}
