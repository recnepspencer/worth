//! Actual physical ownership and owner-authored frames for the Signal port court.
use std::{path::Path, sync::Arc};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{
    ApplicationBuildIdentity, DurableSignalAdmissionIdentity, DurableSignalBranchIdentity,
    DurableSignalCellIncarnation, DurableSignalGeneration, DurableSignalOwnerIdentity,
    DurableSignalSnapshotIdentity, HomeIdentity, SignalBranchAdvanceIdentity,
    SignalDefinitionTransitionIdentity, SignalOwnerSlotIdentity,
};
use worth_signal::facade::durability::{
    SignalAuthoredBranchChange, SignalBranchAdvanceArtifact, SignalBridgeAuthoredSection,
    SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor, SignalDurableHomeBinding,
    SignalDurableMedium,
};
use worth_signal::facade::SignalGraph;
use worth_store::physical_runtime::{
    PhysicalDurabilityCloseoutOutcome, PhysicalReadProtectionDisposition, ServingPhysicalRuntime,
};
use worth_store_application::StoreSignalMedium;

pub(super) fn owner() -> DurableSignalOwnerIdentity {
    DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    )
}
pub(super) fn open(
    root: &Path,
    fresh: bool,
    execution: &ExecutionRequest<'_, '_>,
) -> (Arc<ServingPhysicalRuntime>, Arc<StoreSignalMedium>) {
    open_with_identity(
        root,
        fresh,
        if fresh { 1 } else { 2 },
        "11111111-1111-4111-8111-111111111111",
        execution,
    )
}
pub(super) fn open_with_identity(
    root: &Path,
    fresh: bool,
    incarnation: u64,
    build: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> (Arc<ServingPhysicalRuntime>, Arc<StoreSignalMedium>) {
    let (serving, placement) = super::physical_open::open(root, fresh);
    let binding = SignalDurableHomeBinding::from_admitted_adapter(
        owner(),
        worth_foundational::facade::HomeIncarnation::from_recorded(incarnation).unwrap(),
        [8; 32],
        ApplicationBuildIdentity::parse(build).unwrap(),
    );
    StoreSignalMedium::from_serving(serving, binding, placement, execution).unwrap()
}
pub(super) fn close(
    serving: Arc<ServingPhysicalRuntime>,
    medium: Arc<StoreSignalMedium>,
    execution: &ExecutionRequest<'_, '_>,
) {
    medium.retire(execution).unwrap();
    drop(medium);
    let serving = Arc::try_unwrap(serving).unwrap_or_else(|held| {
        panic!(
            "Signal transport keepers remain: {} serving shares",
            Arc::strong_count(&held)
        )
    });
    let closed = serving.close();
    assert_eq!(
        closed.read_protection().disposition(),
        PhysicalReadProtectionDisposition::Released
    );
    assert!(matches!(
        closed.durability_closeout(),
        PhysicalDurabilityCloseoutOutcome::RecoveryHandoff(_)
    ));
}
pub(super) fn definition(
    execution: &ExecutionRequest<'_, '_>,
    ordinal: u64,
    predecessor: Option<&SignalDefinitionTransitionArtifact>,
) -> SignalDefinitionTransitionArtifact {
    let mut graph = Box::new(SignalGraph::new());
    graph.create_node();
    if predecessor.is_some() {
        graph.create_node();
    }
    let branch = DurableSignalBranchIdentity::from_recorded(owner(), 5);
    let image = graph.capture_authored_image(execution, branch).unwrap();
    let vocabulary =
        worth_signal::facade::durability::SignalDurableTierVocabulary::declare([((), "base")])
            .unwrap();
    let runtime = worth_signal::facade::runtime::RuntimeConfig::<()>::new()
        .capture_authored_image(&graph, &vocabulary, branch, execution)
        .unwrap();
    let payload = b"opaque Bridge-owned source section";
    let mut bytes = ExecutionRequestArrayBuilder::allocate(*execution, payload.len()).unwrap();
    bytes.extend_from_slice(payload).unwrap();
    let bridge = SignalBridgeAuthoredSection::from_bridge_bytes(
        bytes.seal().unwrap().into_immutable_bytes(),
    )
    .unwrap();
    SignalDefinitionTransitionArtifact::capture(
        execution,
        SignalDefinitionTransitionIdentity::from_recorded(owner(), ordinal),
        DurableSignalBranchIdentity::from_recorded(owner(), 5),
        predecessor.map(|p| p.reference()),
        &image,
        &runtime,
        &[],
        &bridge,
    )
    .unwrap()
}
pub(super) fn advance(
    execution: &ExecutionRequest<'_, '_>,
    definition: &SignalDefinitionTransitionArtifact,
) -> SignalBranchAdvanceArtifact {
    let branch = DurableSignalBranchIdentity::from_recorded(owner(), 5);
    let cell = DurableSignalCellIncarnation::from_recorded(branch, 7);
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(cell, 0),
        Some(DurableSignalSnapshotIdentity::from_recorded(owner(), 11)),
        DurableSignalAdmissionIdentity::from_recorded(owner(), 13),
        definition.reference(),
    )
    .unwrap();
    SignalBranchAdvanceArtifact::capture(
        execution,
        SignalBranchAdvanceIdentity::from_recorded(owner(), 17),
        descriptor,
        None,
        SignalAuthoredBranchChange::Created {
            name: "Signal transport branch".into(),
            parent: None,
        },
    )
    .unwrap()
}

pub(super) fn serial_request() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    )
}
