//! Recorded bounds cross real owner readmission before the actual allocator issues nodes.
use super::{drive, medium_fixture as media, owner_fixture as fixture};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_signal::facade::*;

fn cold_source_graph(
    image: &SignalAuthoredImage,
    execution: &ExecutionRequest<'_, '_>,
) -> Box<SignalGraph> {
    let mut fresh = Box::new(SignalGraph::new());
    fresh
        .claim_aspect_lowering_owner(&SignalAspectLoweringOwner::fresh())
        .unwrap();
    fresh
        .readmit_authored_image(execution, image)
        .unwrap()
        .into_parts()
        .0
}

fn readmit_with_nodes(
    medium: Arc<dyn SignalDurableMedium>,
    branch: DurableSignalBranchIdentity,
    rows: &[(u32, u32)],
    execution: &ExecutionRequest<'_, '_>,
) -> fixture::Owner {
    let mut contribution = SignalRecordedIdentityContribution::empty(branch.owner());
    for &(index, generation) in rows {
        contribution
            .observe_node(
                DurableSignalNodeIdentity::from_recorded(branch, index, generation),
                execution,
            )
            .unwrap();
    }
    fixture::program()
        .0
        .readmit_authored_owner(
            medium,
            contribution,
            fixture::vocabulary(),
            &mut worth_signal::facade::durability::SignalOnlyOwnerInstallation,
            execution,
        )
        .unwrap()
}

fn allocate_after_recorded_bounds(rows: impl FnOnce(u32) -> Vec<(u32, u32)>) -> (u32, NodeId) {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut reference = fixture::seed(medium.as_ref(), &execution);
    let (&branch, original) = reference.heads.first_key_value().unwrap();
    let handle = reference
        .memory
        .branch_handle(SignalBranchId(branch.ordinal()))
        .unwrap();
    reference.memory.switch_branch(handle).unwrap();
    let recipe = fixture::recipe(&mut reference.memory);
    let original_live_count = reference.memory.graph().active_node_count();
    let next_index = reference.memory.graph_mut().create_node().index();
    let mut owner = readmit_with_nodes(medium.clone(), branch, &rows(next_index), &execution);
    let mut expected_live = vec![reference.node];
    expected_live.extend(
        owner
            .take_installation_contracts()
            .remove(&branch)
            .unwrap()
            .into_iter()
            .map(|row| row.installed().node()),
    );
    expected_live.sort();
    expected_live.dedup();
    assert_eq!(expected_live.len(), original_live_count);
    let basis = owner.source_port().readmit_descriptor(original).unwrap();
    let (graph, runtime) = owner
        .source_port()
        .capture_authored_images(&basis, &execution)
        .unwrap();
    assert_eq!(graph.as_bytes(), reference.images[&branch].0);
    assert_eq!(runtime.as_bytes(), reference.images[&branch].1);
    let mut issued = None;
    let task = owner
        .mutation_port()
        .begin_advance(execution, &basis, &mut (), |tx| {
            issued = Some(recipe.keyed("after-recorded-bound").node_in_transaction(tx));
            Ok(())
        })
        .unwrap();
    let output = fixture::finish(task, &execution);
    let (image, _) = owner
        .source_port()
        .capture_authored_images(output.branch(), &execution)
        .unwrap();
    let actual = cold_source_graph(&image, &execution);
    expected_live.push(issued.unwrap());
    assert_eq!(
        actual.active_node_count(),
        expected_live.len(),
        "retired placeholders never become live nodes"
    );
    for node in expected_live {
        assert!(
            actual.is_alive(node),
            "every original or issued node remains live"
        );
    }
    drop(output);
    drop((basis, owner));
    media::close(serving, medium, &execution);
    (next_index, issued.unwrap())
}

#[test]
fn recovered_node_bounds_change_future_issuance_without_changing_authored_source() {
    let (index, issued) = allocate_after_recorded_bounds(|index| vec![(index, 7)]);
    assert_eq!((issued.index(), issued.generation()), (index, 8));
}

#[test]
fn exhausted_recorded_node_slots_are_retired_only_when_allocation_reaches_them() {
    let (index, issued) =
        allocate_after_recorded_bounds(|index| vec![(index, u32::MAX), (index + 1, 12)]);
    assert_eq!((issued.index(), issued.generation()), (index + 1, 13));
}

#[test]
fn unused_body_vacancies_and_retired_slots_bound_future_node_issuance() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut reference = fixture::seed(medium.as_ref(), &execution);
    let branches = reference.heads.keys().copied().collect::<Vec<_>>();
    let source_branch = branches[0];
    let target_branch = branches[1];
    let source_head = reference.heads[&source_branch].clone();
    let target_head = reference.heads[&target_branch].clone();
    let handle = reference
        .memory
        .branch_handle(SignalBranchId(source_branch.ordinal()))
        .unwrap();
    reference.memory.switch_branch(handle).unwrap();
    let recipe = fixture::recipe(&mut reference.memory);
    let exhausted = reference.memory.graph_mut().create_node().index();
    let owner = readmit_with_nodes(
        medium.clone(),
        source_branch,
        &[(exhausted, u32::MAX)],
        &execution,
    );
    let basis = owner
        .source_port()
        .readmit_descriptor(&source_head)
        .unwrap();
    let mut original_node = None;
    let task = owner
        .mutation_port()
        .begin_advance(execution, &basis, &mut (), |tx| {
            original_node = Some(
                recipe
                    .keyed("produces-real-retired-slot")
                    .node_in_transaction(tx),
            );
            Ok(())
        })
        .unwrap();
    let output = fixture::finish(task, &execution);
    assert_eq!(original_node.unwrap().index(), exhausted + 1);
    let source = owner
        .read_definition_source(output.artifact().descriptor(), &execution)
        .unwrap();
    let previous = owner
        .read_definition_source(&target_head, &execution)
        .unwrap();
    let mut unused = cold_source_graph(&source.authored_image(), &execution);
    let config = Box::new(
        worth_signal::facade::runtime::RuntimeConfig::readmit_authored_image(
            &execution,
            &source.authored_runtime_image(),
            &unused,
            &fixture::vocabulary(),
        )
        .unwrap(),
    );
    // Public retirement produces a genuine non-tail vacancy. The exhausted
    // placeholder above was produced by the actual canonical allocator, with
    // its wrapped numeric generation and permanent retirement marker intact.
    let vacancy = unused.create_node();
    unused.create_node();
    unused.unregister_node(vacancy).unwrap();
    let image = unused
        .capture_authored_image(&execution, target_branch)
        .unwrap();
    let runtime = config
        .capture_authored_image(&unused, &fixture::vocabulary(), target_branch, &execution)
        .unwrap();
    let empty = ExecutionRequestArrayBuilder::allocate(execution, 0)
        .unwrap()
        .seal()
        .unwrap()
        .into_immutable_bytes();
    assert!(previous.bridge_bytes().is_empty());
    let artifact = SignalDefinitionTransitionArtifact::capture(
        &execution,
        SignalDefinitionTransitionIdentity::from_recorded(target_branch.owner(), 99),
        target_branch,
        Some(target_head.definition()),
        &image,
        &runtime,
        &previous.conditional_occurrences().collect::<Vec<_>>(),
        &SignalBridgeAuthoredSection::from_bridge_bytes(empty).unwrap(),
    )
    .unwrap();
    drop((output, basis, owner));
    assert!(matches!(
        drive(|| medium.advance_definition(&artifact, &execution)).fate(),
        DurablePublicationFate::Committed
    ));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open(directory.path(), false, &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let basis = owner
        .source_port()
        .readmit_descriptor(&target_head)
        .unwrap();
    let (graph, runtime) = owner
        .source_port()
        .capture_authored_images(&basis, &execution)
        .unwrap();
    assert_eq!(graph.as_bytes(), reference.images[&target_branch].0);
    assert_eq!(runtime.as_bytes(), reference.images[&target_branch].1);
    let mut issued = None;
    let task = owner
        .mutation_port()
        .begin_advance(execution, &basis, &mut (), |tx| {
            let first = recipe.keyed("past-unused-retired").node_in_transaction(tx);
            let second = recipe.keyed("past-unused-vacancy").node_in_transaction(tx);
            issued = Some((first, second));
            Ok(())
        })
        .unwrap();
    let (first, second) = issued.unwrap();
    assert_eq!(first.index(), exhausted + 1);
    assert!(first.generation() > original_node.unwrap().generation());
    assert_eq!(second.index(), vacancy.index());
    assert!(second.generation() > vacancy.generation() + 1);
    drop(fixture::finish(task, &execution));
    drop((basis, owner));
    media::close(serving, medium, &execution);
}
