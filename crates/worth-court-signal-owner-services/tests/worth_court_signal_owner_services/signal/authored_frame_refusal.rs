//! Readmission preserves constructor invariants even for checksummed source.
use sha2::{Digest, Sha256};
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    DurableSignalAdmissionIdentity, DurableSignalBranchIdentity, DurableSignalCellIncarnation,
    DurableSignalGeneration, DurableSignalOwnerIdentity, HomeIdentity, SignalBranchAdvanceIdentity,
    SignalDefinitionTransitionIdentity, SignalDefinitionTransitionReference,
    SignalOwnerSlotIdentity,
};
use worth_signal::facade::durability::{
    SignalAuthoredBranchChange, SignalAuthoredImage, SignalAuthoredImageDenial,
    SignalBranchAdvanceArtifact, SignalDurableBranchDescriptor, SignalDurableFrameDenial,
};
use worth_signal::facade::{
    Aspect, DependencyEdge, PartitionSubscription, SignalError, SignalGraph,
};

fn request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    )
}
fn branch() -> DurableSignalBranchIdentity {
    DurableSignalBranchIdentity::from_recorded(
        DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
            SignalOwnerSlotIdentity::from_recorded(1),
        ),
        0,
    )
}
fn reseal(bytes: &mut [u8]) {
    let mut checksum = Sha256::new();
    checksum.update(&bytes[8..24]);
    checksum.update(&bytes[56..]);
    bytes[24..56].copy_from_slice(&checksum.finalize());
}

#[test]
fn branch_decoder_refuses_snapshot_effects_without_a_snapshot() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let branch = branch();
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, 1),
            0,
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(branch.owner(), 1),
        SignalDefinitionTransitionReference::from_recorded(
            SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), 1),
            [9; 32],
        ),
    )
    .unwrap();
    let artifact = SignalBranchAdvanceArtifact::capture(
        &execution,
        SignalBranchAdvanceIdentity::from_recorded(branch.owner(), 1),
        descriptor,
        None,
        SignalAuthoredBranchChange::Advanced,
    )
    .unwrap();
    for tag in [2, 3] {
        let mut bytes = artifact.as_bytes().to_vec();
        *bytes.last_mut().unwrap() = tag;
        reseal(&mut bytes);
        assert!(matches!(
            SignalBranchAdvanceArtifact::readmit_bytes(&execution, &bytes),
            Err(SignalDurableFrameDenial::SnapshotRequired),
        ));
    }
}

#[test]
fn graph_readmission_refuses_stale_declarations_and_declaration_cycles() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let middle = graph.create_node();
    let consumer = graph.create_node();
    let alternate = graph.create_node();
    graph
        .set_dependencies(middle, [DependencyEdge::new(source, Aspect::new(0))])
        .unwrap();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(middle, Aspect::new(0))])
        .unwrap();
    let original = graph.capture_authored_image(&execution, branch()).unwrap();
    graph
        .set_dependencies(middle, [DependencyEdge::new(alternate, Aspect::new(0))])
        .unwrap();
    let changed = graph.capture_authored_image(&execution, branch()).unwrap();
    // Compare two legal images to locate the encoded edge field without
    // depending on offsets within unrelated node policy encodings.
    let offsets = (56..original.as_bytes().len())
        .filter(|offset| original.as_bytes()[*offset] != changed.as_bytes()[*offset])
        .collect::<Vec<_>>();
    let [edge] = offsets.as_slice() else {
        panic!("one source index byte must differ")
    };
    for (offset, value, cycle) in [(*edge, 200, false), (*edge + 4, 1, false), (*edge, 2, true)] {
        let mut bytes = original.as_bytes().to_vec();
        bytes[offset] = value;
        reseal(&mut bytes);
        let image = SignalAuthoredImage::readmit_bytes(&execution, &bytes).unwrap();
        let error = match Box::new(SignalGraph::new()).readmit_authored_image(&execution, &image) {
            Ok(_) => panic!("invalid declaration topology must refuse"),
            Err(error) => error,
        };
        match error {
            SignalAuthoredImageDenial::Topology(error) => {
                if cycle {
                    assert!(matches!(*error, SignalError::CycleDetected { .. }));
                } else {
                    assert!(matches!(*error, SignalError::StaleHandle { .. }));
                }
            }
            error => panic!("expected typed topology refusal, got {error:?}"),
        }
    }
}

#[test]
fn graph_readmission_reconstructs_valid_scoped_declarations() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let consumer = graph.create_node();
    let scope = PartitionSubscription::partition_and_detail("orders", "totals");
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::with_partition_scope(
                source,
                Aspect::new(1),
                scope,
            )],
        )
        .unwrap();
    let image = graph.capture_authored_image(&execution, branch()).unwrap();
    let (recovered, contracts) = Box::new(SignalGraph::new())
        .readmit_authored_image(&execution, &image)
        .unwrap()
        .into_parts();
    assert!(contracts.is_empty());
    assert_eq!(
        recovered.declared_dependencies_of(consumer).unwrap(),
        graph.declared_dependencies_of(consumer).unwrap()
    );
    assert_eq!(
        recovered
            .capture_authored_image(&execution, branch())
            .unwrap()
            .as_bytes(),
        image.as_bytes()
    );
}
