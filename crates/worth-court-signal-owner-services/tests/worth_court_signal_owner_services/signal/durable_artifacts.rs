use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionRequestArrayBuilder, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIdentity, DurableProductBranchIncarnation, DurableSignalAdmissionIdentity,
    DurableSignalBranchIdentity, DurableSignalCellIncarnation, DurableSignalGeneration,
    DurableSignalOwnerIdentity, DurableSignalSnapshotIdentity, DurableWorldOwnerIdentity,
    HomeIdentity, SignalBranchAdvanceIdentity, SignalDefinitionTransitionIdentity,
    SignalDefinitionTransitionReference, SignalOwnerSlotIdentity, WorldOwnerSlotIdentity,
};
use worth_signal::facade::durability::{
    SignalAuthoredBranchChange, SignalBranchAdvanceArtifact, SignalBridgeAuthoredSection,
    SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor, SignalDurableFrameDenial,
};
use worth_signal::facade::SignalGraph;

fn owner(home: u8) -> DurableSignalOwnerIdentity {
    DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([home; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    )
}
fn descriptor(
    owner: DurableSignalOwnerIdentity,
    definition: SignalDefinitionTransitionReference,
) -> SignalDurableBranchDescriptor {
    let branch = DurableSignalBranchIdentity::from_recorded(owner, 5);
    let cell = DurableSignalCellIncarnation::from_recorded(branch, 7);
    SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(cell, 11),
        Some(DurableSignalSnapshotIdentity::from_recorded(owner, 13)),
        DurableSignalAdmissionIdentity::from_recorded(owner, 17),
        definition,
    )
    .unwrap()
}
#[test]
fn definition_transition_retains_exact_authored_image_bridge_section_and_predecessor() {
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    graph.create_node();
    let branch = DurableSignalBranchIdentity::from_recorded(owner(1), 5);
    let image = graph.capture_authored_image(&request, branch).unwrap();
    let vocabulary =
        worth_signal::facade::durability::SignalDurableTierVocabulary::declare([((), "base")])
            .unwrap();
    let runtime = worth_signal::facade::runtime::RuntimeConfig::<()>::new()
        .capture_authored_image(&graph, &vocabulary, branch, &request)
        .unwrap();
    let bridge_bytes = b"bridge-owned-section-fixture";
    let mut bytes = ExecutionRequestArrayBuilder::allocate(request, bridge_bytes.len()).unwrap();
    bytes.extend_from_slice(bridge_bytes).unwrap();
    let bridge = SignalBridgeAuthoredSection::from_bridge_bytes(
        bytes.seal().unwrap().into_immutable_bytes(),
    )
    .unwrap();
    let identity = SignalDefinitionTransitionIdentity::from_recorded(owner(1), 2);
    let previous = SignalDefinitionTransitionReference::from_recorded(
        SignalDefinitionTransitionIdentity::from_recorded(owner(1), 1),
        [19; 32],
    );
    let occurrence =
        worth_foundational::facade::DurableSignalConditionalDefinitionOccurrence::from_recorded(
            worth_foundational::facade::DurableSignalNodeIdentity::from_recorded(branch, 0, 0),
            worth_foundational::facade::DurableSignalConditionalOccurrence::from_recorded(
                owner(1),
                41,
            ),
        )
        .unwrap();
    let artifact = SignalDefinitionTransitionArtifact::capture(
        &request,
        identity,
        branch,
        Some(previous),
        &image,
        &runtime,
        &[occurrence],
        &bridge,
    )
    .unwrap();
    let read =
        SignalDefinitionTransitionArtifact::readmit_bytes(&request, artifact.as_bytes()).unwrap();
    assert_eq!(read.identity(), identity);
    assert_eq!(read.branch(), branch);
    assert_eq!(read.predecessor(), Some(previous));
    assert_eq!(read.image_bytes(), image.as_bytes());
    assert_eq!(read.runtime_bytes(), runtime.as_bytes());
    assert_eq!(read.authored_image().branch(), branch);
    assert_eq!(read.authored_runtime_image().branch(), branch);
    assert_eq!(
        read.conditional_occurrences().collect::<Vec<_>>(),
        [occurrence]
    );
    assert_eq!(read.bridge_bytes(), bridge_bytes);
    assert_eq!(read.reference(), artifact.reference());
    let mut unknown = artifact.as_bytes().to_vec();
    unknown[8..12].copy_from_slice(&2_u32.to_le_bytes());
    assert!(matches!(
        SignalDefinitionTransitionArtifact::readmit_bytes(&request, &unknown),
        Err(SignalDurableFrameDenial::UnsupportedVersion(_))
    ));
}
#[test]
fn initial_branch_has_no_snapshot_and_snapshot_effects_require_one() {
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&funding);
    let owner = owner(1);
    let branch = DurableSignalBranchIdentity::from_recorded(owner, 0);
    let reference = SignalDefinitionTransitionReference::from_recorded(
        SignalDefinitionTransitionIdentity::from_recorded(owner, 1),
        [37; 32],
    );
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, 1),
            0,
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(owner, 1),
        reference,
    )
    .unwrap();
    let id = SignalBranchAdvanceIdentity::from_recorded(owner, 1);
    let original = SignalBranchAdvanceArtifact::capture(
        &request,
        id,
        descriptor.clone(),
        None,
        SignalAuthoredBranchChange::Created {
            name: "main".into(),
            parent: None,
        },
    )
    .unwrap();
    let read = SignalBranchAdvanceArtifact::readmit_bytes(&request, original.as_bytes()).unwrap();
    assert_eq!(read.descriptor(), &descriptor);
    assert_eq!(read.descriptor().snapshot(), None);
    for change in [
        SignalAuthoredBranchChange::SnapshotCaptured,
        SignalAuthoredBranchChange::Restored,
    ] {
        assert!(matches!(
            SignalBranchAdvanceArtifact::capture(&request, id, descriptor.clone(), None, change),
            Err(SignalDurableFrameDenial::SnapshotRequired)
        ));
    }
}
#[test]
fn branch_effect_preserves_original_attempt_association_and_all_recorded_axes() {
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&funding);
    let owner = owner(1);
    let world = DurableWorldOwnerIdentity::from_recorded(
        owner.home(),
        WorldOwnerSlotIdentity::from_recorded(23),
    );
    let association = CompositeAttemptAssociation::from_recorded(
        DurableCompositePublicationAttemptIdentity::from_recorded(world, 29),
        DurableProductBranchIdentity::from_recorded(world, "world-branch".into()).unwrap(),
        DurableProductBranchIncarnation::from_recorded(world, 31),
    )
    .unwrap();
    let definition = SignalDefinitionTransitionReference::from_recorded(
        SignalDefinitionTransitionIdentity::from_recorded(owner, 2),
        [37; 32],
    );
    let descriptor = descriptor(owner, definition);
    let id = SignalBranchAdvanceIdentity::from_recorded(owner, 41);
    for change in [
        SignalAuthoredBranchChange::Created {
            name: "signal-branch".into(),
            parent: None,
        },
        SignalAuthoredBranchChange::Advanced,
        SignalAuthoredBranchChange::SnapshotCaptured,
        SignalAuthoredBranchChange::Restored,
        SignalAuthoredBranchChange::Retired,
    ] {
        let artifact = SignalBranchAdvanceArtifact::capture(
            &request,
            id,
            descriptor.clone(),
            Some(association.clone()),
            change.clone(),
        )
        .unwrap();
        let read =
            SignalBranchAdvanceArtifact::readmit_bytes(&request, artifact.as_bytes()).unwrap();
        assert_eq!(read.identity(), id);
        assert_eq!(read.descriptor(), &descriptor);
        assert_eq!(read.association(), Some(&association));
        assert_eq!(read.change(), &change);
    }
}
#[test]
fn foreign_definition_owner_and_foreign_attempt_home_refuse_as_typed_data() {
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&funding);
    let definition = SignalDefinitionTransitionReference::from_recorded(
        SignalDefinitionTransitionIdentity::from_recorded(owner(2), 1),
        [3; 32],
    );
    let cell = DurableSignalCellIncarnation::from_recorded(
        DurableSignalBranchIdentity::from_recorded(owner(1), 5),
        7,
    );
    assert!(matches!(
        SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(cell, 11),
            Some(DurableSignalSnapshotIdentity::from_recorded(owner(1), 13)),
            DurableSignalAdmissionIdentity::from_recorded(owner(1), 17),
            definition
        ),
        Err(SignalDurableFrameDenial::ForeignOwner)
    ));
    let descriptor = descriptor(owner(2), definition);
    assert!(matches!(
        SignalBranchAdvanceArtifact::capture(
            &request,
            SignalBranchAdvanceIdentity::from_recorded(owner(1), 1),
            descriptor.clone(),
            None,
            SignalAuthoredBranchChange::Advanced
        ),
        Err(SignalDurableFrameDenial::ForeignOwner)
    ));
    let world = DurableWorldOwnerIdentity::from_recorded(
        owner(1).home(),
        WorldOwnerSlotIdentity::from_recorded(23),
    );
    let association = CompositeAttemptAssociation::from_recorded(
        DurableCompositePublicationAttemptIdentity::from_recorded(world, 29),
        DurableProductBranchIdentity::from_recorded(world, "world-branch".into()).unwrap(),
        DurableProductBranchIncarnation::from_recorded(world, 31),
    )
    .unwrap();
    assert!(matches!(
        SignalBranchAdvanceArtifact::capture(
            &request,
            SignalBranchAdvanceIdentity::from_recorded(owner(2), 1),
            descriptor,
            Some(association),
            SignalAuthoredBranchChange::Advanced,
        ),
        Err(SignalDurableFrameDenial::ForeignOwner)
    ));
}

mod source_point;
