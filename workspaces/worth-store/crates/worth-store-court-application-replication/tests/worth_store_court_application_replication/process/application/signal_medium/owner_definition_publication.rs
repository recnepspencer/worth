//! Authored changes publish bodies, while unused bodies burn recorded incarnations.
use super::{drive, medium_fixture as media, owner_fixture as fixture};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_signal::facade::*;

#[test]
fn runtime_published_keyed_definitions_reopen_without_reissuing_unused_body_incarnations() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut reference = fixture::seed(medium.as_ref(), &execution);
    let (&branch, original) = reference.heads.first_key_value().unwrap();
    let original = original.clone();
    let source = medium.capture_recovery_source(&execution).unwrap();
    let bytes = source
        .read(SignalDurableJournal::Definitions, 2, &execution)
        .unwrap();
    let previous =
        SignalDefinitionTransitionArtifact::readmit_bytes(&execution, bytes.bytes()).unwrap();
    drop(source);
    let handle = reference
        .memory
        .branch_handle(SignalBranchId(branch.ordinal()))
        .unwrap();
    reference.memory.switch_branch(handle).unwrap();
    let recipe = fixture::recipe(&mut reference.memory);
    let unused_node = recipe.keyed("unused-body-key").node(&mut reference.memory);
    let image = reference
        .memory
        .graph()
        .capture_authored_image(&execution, branch)
        .unwrap();
    let runtime = reference
        .memory
        .config()
        .capture_authored_image(
            reference.memory.graph(),
            &fixture::vocabulary(),
            branch,
            &execution,
        )
        .unwrap();
    let unused = SignalDefinitionTransitionArtifact::capture(
        &execution,
        SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), 99),
        branch,
        Some(original.definition()),
        &image,
        &runtime,
        &previous.conditional_occurrences().collect::<Vec<_>>(),
        &SignalBridgeAuthoredSection::from_bridge_bytes({
            let mut bytes =
                ExecutionRequestArrayBuilder::allocate(execution, previous.bridge_bytes().len())
                    .unwrap();
            bytes.extend_from_slice(previous.bridge_bytes()).unwrap();
            bytes.seal().unwrap().into_immutable_bytes()
        })
        .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        drive(|| medium.advance_definition(&unused, &execution)).fate(),
        DurablePublicationFate::Committed
    ));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open(directory.path(), false, &execution);
    let mut contribution = SignalRecordedIdentityContribution::empty(branch.owner());
    contribution
        .observe_advance(SignalBranchAdvanceIdentity::from_recorded(
            branch.owner(),
            77,
        ))
        .unwrap();
    contribution
        .observe_definition(SignalDefinitionTransitionIdentity::from_recorded(
            branch.owner(),
            120,
        ))
        .unwrap();
    let recorded_open = SignalDurableBranchDescriptor::from_recorded(
        original.generation(),
        original.snapshot(),
        DurableSignalAdmissionIdentity::from_recorded(branch.owner(), 66),
        original.definition(),
    )
    .unwrap();
    contribution.observe_descriptor(&recorded_open).unwrap();
    let owner = fixture::program()
        .0
        .readmit_authored_owner(
            medium.clone(),
            contribution,
            fixture::vocabulary(),
            &execution,
        )
        .unwrap();
    let basis = owner.source_port().readmit_descriptor(&original).unwrap();
    let (graph, runtime) = owner
        .source_port()
        .capture_authored_images(&basis, &execution)
        .unwrap();
    assert_eq!(
        graph.as_bytes(),
        reference.images[&branch].0,
        "unused body cannot replace canonical definitions"
    );
    assert_eq!(runtime.as_bytes(), reference.images[&branch].1);
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let mut issued = None;
    let task = owner
        .mutation_port()
        .begin_advance(execution, &basis, &mut (), |tx| {
            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            issued = Some(recipe.keyed("published-key").node_in_transaction(tx));
            Ok(())
        })
        .unwrap();
    let issued = issued.unwrap();
    assert_eq!(issued.index(), unused_node.index());
    assert!(issued.generation() > unused_node.generation());
    assert_eq!(task.original().ordinal(), 78);
    let fresh_funding = media::serial_request();
    let output = fixture::finish(task, &ExecutionRequest::serial(&fresh_funding));
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    assert_eq!(output.artifact().descriptor().admission().ordinal(), 67);
    assert_eq!(
        output
            .artifact()
            .descriptor()
            .definition()
            .identity()
            .ordinal(),
        121
    );
    let next = output.artifact().descriptor().clone();
    let source = owner.read_definition_source(&next, &execution).unwrap();
    assert_eq!(source.predecessor(), Some(original.definition()));
    assert_eq!(
        source.conditional_occurrences().collect::<Vec<_>>(),
        previous.conditional_occurrences().collect::<Vec<_>>()
    );
    let (graph, runtime) = owner
        .source_port()
        .capture_authored_images(output.branch(), &execution)
        .unwrap();
    let expected = (graph.as_bytes().to_vec(), runtime.as_bytes().to_vec());
    drop((output, basis, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        3,
        "11111111-1111-4111-8111-111111111111",
        &execution,
    );
    let owner = fixture::readmit(medium.clone(), &execution);
    let basis = owner.source_port().readmit_descriptor(&next).unwrap();
    let (graph, runtime) = owner
        .source_port()
        .capture_authored_images(&basis, &execution)
        .unwrap();
    assert_eq!(graph.as_bytes(), expected.0);
    assert_eq!(runtime.as_bytes(), expected.1);
    let mut same_node = None;
    let task = owner
        .mutation_port()
        .begin_advance(execution, &basis, &mut (), |tx| {
            same_node = Some(recipe.keyed("published-key").node_in_transaction(tx));
            Ok(())
        })
        .unwrap();
    assert_eq!(same_node, Some(issued));
    drop(fixture::finish(task, &execution));
    drop((basis, owner));
    media::close(serving, medium, &execution);
}
