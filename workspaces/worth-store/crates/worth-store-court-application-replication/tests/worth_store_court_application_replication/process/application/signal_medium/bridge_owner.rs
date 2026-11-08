//! Bridge cold installation crosses real Store journals, without a Home or mixed claim.
use super::{medium_fixture as media, owner_fixture};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly;
use worth_signal::facade::durability::*;
use worth_signal::facade::*;
mod derived_reference;
mod program;
mod quiescent_kill;
mod source;

fn readmit(
    medium: Arc<dyn SignalDurableMedium>,
    execution: &ExecutionRequest<'_, '_>,
) -> Box<BridgeReadmittedAuthoredAssembly> {
    let mut bindings = program::Bindings { providers: 0 };
    let owner = program::program()
        .builder
        .readmit_authored_owner(
            medium,
            SignalRecordedIdentityContribution::empty(media::owner()),
            &mut bindings,
            execution,
        )
        .unwrap();
    assert_eq!(
        bindings.providers, 6,
        "each branch rebuilds both provider sets"
    );
    owner
}
fn assert_sources(
    owner: &BridgeReadmittedAuthoredAssembly,
    reference: &source::Reference,
    execution: &ExecutionRequest<'_, '_>,
    demands: u64,
) {
    let port = owner.signal_owner().source_port();
    let branches = port.branches().unwrap();
    assert_eq!(branches.len(), 3);
    for (((branch, handle), definition), original) in branches
        .iter()
        .zip(&reference.definitions)
        .zip(&reference.advances)
    {
        assert_eq!(handle.name, format!("bridge-branch-{}", branch.ordinal()));
        assert_eq!(
            handle.parent_branch_id.map(|id| id.0),
            branch.ordinal().checked_sub(1).filter(|id| *id != 0)
        );
        let descriptor = port.observe(*branch).unwrap();
        assert_eq!(
            descriptor.generation().ordinal(),
            original.descriptor().generation().ordinal() + demands
        );
        assert_eq!(descriptor.definition(), definition.reference());
        let basis = port.readmit_descriptor(&descriptor).unwrap();
        let (graph, runtime) = port.capture_authored_images(&basis, execution).unwrap();
        assert_eq!(graph.as_bytes(), definition.image_bytes());
        assert_eq!(runtime.as_bytes(), definition.runtime_bytes());
        assert_eq!(
            owner
                .authored_section(*branch)
                .unwrap()
                .as_signal_section()
                .as_bytes(),
            definition.bridge_bytes()
        );
        let actual: Vec<_> = owner.conditional_definitions(*branch).collect();
        assert_eq!(actual.len(), reference.program.lowerings.len());
        for (actual, expected) in actual.iter().zip(&reference.program.lowerings) {
            assert_eq!(actual.contract(), expected.contract());
            assert_eq!(actual.location(), expected.location());
            assert_eq!(actual.signal_node(), expected.signal_node());
            assert_eq!(
                actual.correspondence_count(),
                expected.correspondence_count()
            );
        }
        let retention = owner.conditional_retention(*branch).unwrap();
        assert_eq!(retention.retained_definition_candidates(), 0);
        assert_eq!(retention.managed_clocks(), 0);
        assert_eq!(retention.reserved_temporal_intents(), 0);
        assert!(retention.retained_bytes() > 0);
    }
}
fn demand(
    owner: &BridgeReadmittedAuthoredAssembly,
    node: NodeId,
    execution: &ExecutionRequest<'_, '_>,
    calls: &AtomicUsize,
    expected: AspectVersion,
) {
    for (branch, _) in owner.signal_owner().source_port().branches().unwrap() {
        let port = owner.signal_owner().source_port();
        let descriptor = port.observe(branch).unwrap();
        let basis = port.readmit_descriptor(&descriptor).unwrap();
        let task = owner
            .signal_owner()
            .mutation_port()
            .begin_advance(*execution, &basis, &mut (), |tx| {
                let value = tx.target(node).read(
                    &|_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
                        calls.fetch_add(1, Ordering::Relaxed);
                        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
                    },
                )?;
                assert_eq!(value, expected);
                Ok(())
            })
            .unwrap();
        let output = owner_fixture::finish(task, execution);
        assert_eq!(
            output.artifact().descriptor().definition(),
            descriptor.definition(),
            "derived demand must not publish a definition"
        );
    }
}
#[test]
fn bridge_definitions_reopen_from_store_with_cold_values_under_same_and_changed_builds() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let expected = derived_reference::value(&execution);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit(medium.clone(), &execution);
    assert_sources(&owner, &reference, &execution, 0);
    let calls = AtomicUsize::new(0);
    demand(
        &owner,
        reference.program.value,
        &execution,
        &calls,
        expected.clone(),
    );
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    drop(owner);
    media::close(serving, medium, &execution);
    for (incarnation, build) in [
        (2, "11111111-1111-4111-8111-111111111111"),
        (3, "22222222-2222-4222-8222-222222222222"),
    ] {
        let (serving, medium) =
            media::open_with_identity(directory.path(), false, incarnation, build, &execution);
        let owner = readmit(medium.clone(), &execution);
        assert_sources(&owner, &reference, &execution, 1 + (incarnation - 2) * 2);
        let before = calls.load(Ordering::Relaxed);
        demand(
            &owner,
            reference.program.value,
            &execution,
            &calls,
            expected.clone(),
        );
        assert_eq!(
            calls.load(Ordering::Relaxed),
            before + 3,
            "every reopened branch starts cold"
        );
        demand(
            &owner,
            reference.program.value,
            &execution,
            &calls,
            expected.clone(),
        );
        assert_eq!(
            calls.load(Ordering::Relaxed),
            before + 3,
            "serving derived cache is reusable"
        );
        assert_sources(&owner, &reference, &execution, 3 + (incarnation - 2) * 2);
        drop(owner);
        media::close(serving, medium, &execution);
    }
}
