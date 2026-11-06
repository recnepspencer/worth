//! Value/backing courts: these records describe no serving Home or committed effect.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, HomeIdentity, WorldOwnerSlotIdentity,
};

fn association() -> CompositeAttemptAssociation {
    let owner = DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(0),
    );
    CompositeAttemptAssociation::from_recorded(
        DurableCompositePublicationAttemptIdentity::from_recorded(owner, 1),
        DurableProductBranchIdentity::from_recorded(owner, "n".repeat(256)).unwrap(),
        DurableProductBranchIncarnation::from_recorded(owner, 1),
    )
    .unwrap()
}

#[test]
fn sharing_a_world_artifact_keeps_its_names_and_frame_on_the_original_memory_grant() {
    let association = association();
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&policy);
    // A direct encoder control measures its actual retained charge, including
    // any framework granule. The artifact must ALSO fund its separate names.
    let record = WorldRecord::Released {
        custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
            association.attempt().owner(),
            1,
        ),
        association: association.clone(),
        relational: None,
    };
    let frame = super::super::codec::encode(&record, &request)
        .unwrap()
        .into_immutable_bytes();
    let frame_charge = budget.charged_memory_bytes();
    let original = WorldDurableArtifact::from_input(
        WorldArtifactInput::Released {
            custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
                association.attempt().owner(),
                1,
            ),
            association: &association,
            relational: None,
        },
        &request,
    )
    .unwrap();
    let charged = budget.charged_memory_bytes();
    assert_eq!(original.as_bytes(), frame.bytes());
    assert!(
        charged >= 2 * frame_charge + association.branch().name().len() as u64,
        "the frame's existing grant cannot fund the separately retained name"
    );
    let shared = original.clone();
    assert!(shared.same_subject(&original));
    assert_eq!(budget.charged_memory_bytes(), charged);
    drop(original);
    assert_eq!(budget.charged_memory_bytes(), charged);
    assert_eq!(shared.as_bytes(), frame.bytes());
    drop(shared);
    assert_eq!(budget.charged_memory_bytes(), frame_charge);
    drop(frame);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn a_world_artifact_memory_refusal_precedes_name_and_frame_construction_without_leaking_a_grant() {
    let association = association();
    let budget = SerialMemoryBudget::new(Some(0));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let refused = WorldDurableArtifact::from_input(
        WorldArtifactInput::Released {
            custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
                association.attempt().owner(),
                1,
            ),
            association: &association,
            relational: None,
        },
        &ExecutionRequest::serial(&policy),
    );
    assert!(matches!(
        refused,
        Err(crate::durability::WorldDurableMediumDenial::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(
                    worth_execution::MemoryLimitDenial::Capacity { admitted: 0, .. }
                )
            )
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn world_artifact_name_copy_uses_the_reviewed_committed_compiler() {
    let toolchain = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-toolchain.toml"),
    )
    .unwrap();
    assert_eq!(
        toolchain.lines().find(|line| line.starts_with("channel")),
        Some("channel = \"1.99.1\"")
    );
}
