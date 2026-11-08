//! Inherited conditional sources get new branch occurrences and derived projections.
use super::*;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::branch::*;

#[test]
fn bridge_durable_fork_rebuilds_conditional_projection_before_ack_and_reopens_it_cold() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit(medium.clone(), &execution);
    let parent = reference.advances[0].descriptor();
    let port = owner.signal_owner().source_port();
    let source = port.readmit_descriptor(parent).unwrap();
    let inherited = owner
        .with_authored_branch(parent.branch(), |view| {
            view.conditional_definitions().collect::<Vec<_>>()
        })
        .unwrap();
    let reservation = owner
        .signal_owner()
        .mutation_port()
        .reserve_fork(
            &execution,
            &source,
            validate_signal_branch_name("bridge-durable-child").unwrap(),
        )
        .unwrap();
    let target = reservation.target();
    let task = owner
        .begin_authored_branch_fork(
            execution,
            reservation,
            &SignalOwnerCancellationSource::new().token(),
        )
        .unwrap();
    let original = task.original();
    assert!(matches!(owner.with_authored_branch(target, |_| ()),
        Err(BridgeAuthoredBranchDenial::ForkReserved { branch }) if branch == target));
    assert!(
        matches!(owner.with_authored_branch(parent.branch(), |_| ()),
        Err(BridgeAuthoredBranchDenial::Pending { original: held, .. }) if held == original)
    );
    drop(task);
    let output = owner_fixture::finish(
        owner
            .signal_owner()
            .mutation_port()
            .resume_original(original)
            .unwrap(),
        &execution,
    );
    assert!(matches!(
        output.effect(),
        SignalDurableOwnerEffect::Forked(_)
    ));
    assert_eq!(output.evidence().position(), 4);
    let section = owner
        .with_authored_branch(target, |view| {
            assert_eq!(
                view.conditional_retention()
                    .retained_definition_candidates(),
                0
            );
            assert_eq!(view.conditional_retention().managed_clocks(), 0);
            assert_eq!(view.conditional_retention().reserved_temporal_intents(), 0);
            let actual = view.conditional_definitions().collect::<Vec<_>>();
            assert_eq!(actual.len(), inherited.len());
            for (actual, expected) in actual.iter().zip(&inherited) {
                assert_eq!(actual.contract(), expected.contract());
                assert_eq!(actual.location(), expected.location());
                assert_eq!(actual.signal_node(), expected.signal_node());
                assert_eq!(
                    actual.correspondence_count(),
                    expected.correspondence_count()
                );
                assert_ne!(actual.projection(), expected.projection());
            }
            view.authored_section()
                .as_signal_section()
                .as_bytes()
                .to_vec()
        })
        .unwrap();
    owner
        .with_authored_branch(parent.branch(), |view| {
            assert!(view
                .conditional_definitions()
                .zip(&inherited)
                .all(|(actual, expected)| Arc::ptr_eq(&actual, expected)));
        })
        .unwrap();
    let definition = owner
        .signal_owner()
        .read_definition_source(output.branch().descriptor(), &execution)
        .unwrap();
    assert_eq!(definition.bridge_bytes(), section);
    assert_eq!(definition.predecessor(), Some(parent.definition()));
    let target_occurrences = definition.conditional_occurrences().collect::<Vec<_>>();
    assert_eq!(target_occurrences.len(), 2);
    for occurrence in &target_occurrences {
        assert_eq!(occurrence.node().branch(), target);
        assert!(reference
            .definitions
            .iter()
            .flat_map(|definition| definition.conditional_occurrences())
            .all(|source| source.occurrence() != occurrence.occurrence()));
    }
    let descriptor = output.branch().descriptor().clone();
    drop((definition, output, inherited, source, port, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        2,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let mut bindings = program::Bindings { providers: 0 };
    let owner = program::program()
        .builder
        .readmit_authored_owner(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            &mut bindings,
            &execution,
        )
        .unwrap();
    assert_eq!(bindings.providers, 8);
    assert_eq!(
        owner.signal_owner().source_port().observe(target).unwrap(),
        descriptor
    );
    owner
        .with_authored_branch(target, |view| {
            assert_eq!(
                view.authored_section().as_signal_section().as_bytes(),
                section
            );
            assert_eq!(view.conditional_definitions().count(), 2);
        })
        .unwrap();
    let calls = AtomicUsize::new(0);
    demand(
        &owner,
        reference.program.value,
        &execution,
        &calls,
        derived_reference::value(&execution),
    );
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    drop(owner);
    media::close(serving, medium, &execution);
}
