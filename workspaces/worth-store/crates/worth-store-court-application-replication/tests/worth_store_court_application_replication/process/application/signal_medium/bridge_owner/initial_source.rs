//! Initial authored publication uses the real program and both original Store ACKs.
use super::*;
use std::time::{Duration, Instant};
use worth_foundational::facade::{DurablePublicationFate, SignalDurableJournal};
use worth_runtime_bridge::facade::*;
mod paused_medium;

fn finish(pending: &mut BridgeInitialSourcePublication, execution: &ExecutionRequest<'_, '_>) {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        match pending.advance(execution).unwrap() {
            SignalInitialSourceProgress::Published => return,
            SignalInitialSourceProgress::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
        }
    }
}
fn prepare(
    builder: Box<BridgeConditionalRuntimeBuilder>,
    medium: Arc<dyn SignalDurableMedium>,
    execution: &ExecutionRequest<'_, '_>,
) -> BridgeInitialSourcePublication {
    builder
        .prepare_initial_authored_source(
            medium,
            SignalRecordedIdentityContribution::empty(media::owner()),
            execution,
        )
        .unwrap_or_else(|refusal| panic!("initial preparation: {:?}", refusal.cause()))
}
fn published(pending: BridgeInitialSourcePublication) -> BridgePublishedInitialSource {
    pending
        .try_into_published()
        .unwrap_or_else(|_| panic!("original has not committed"))
}
#[test]
fn initial_bridge_source_retains_both_originals_and_readmits_cold_under_a_changed_build() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let expected = derived_reference::value(&execution);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let observed = paused_medium::PausedMedium::new(medium.clone());
    let program = program::program();
    let value = program.value;
    let mut pending = prepare(program.builder, observed.clone(), &execution);
    let definition = pending.definition().clone();
    let original = pending.advance_artifact().clone();
    assert_eq!(definition.branch().ordinal(), 0);
    assert_eq!(definition.conditional_occurrences().count(), 2);
    assert!(definition.predecessor().is_none());
    assert!(matches!(
        original.change(),
        SignalAuthoredBranchChange::Created { parent: None, .. }
    ));
    assert!(original.association().is_none());
    let denied = program::program().builder.prepare_initial_authored_source(
        observed.clone(),
        SignalRecordedIdentityContribution::empty(media::owner()),
        &execution,
    );
    assert!(matches!(denied.err().unwrap().cause(),
        BridgeInitialSourceDenial::Signal(cause) if matches!(cause.as_ref(),
            SignalInitialSourceDenial::Medium(SignalDurableMediumDenial::OwnerAlreadyReadmitted(_)))));
    let deadline = Instant::now() + Duration::from_secs(120);
    while !observed.branch_observed.load(Ordering::Relaxed) {
        assert_eq!(
            pending.advance(&execution).unwrap(),
            SignalInitialSourceProgress::Pending
        );
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        1
    );
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        0
    );
    drop(source);
    pending = pending
        .try_into_published()
        .err()
        .expect("pending original must be returned");
    let cancellation = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    observed.pause_branch.store(false, Ordering::Relaxed);
    let refusal = pending
        .advance(&ExecutionRequest::serial(&stopped))
        .unwrap_err();
    assert_eq!(refusal.journal, SignalDurableJournal::Advances);
    assert!(matches!(
        refusal.fate,
        DurablePublicationFate::ProvenNoEffect {
            detail: SignalDurableAdvanceDenial::Medium(SignalDurableMediumDenial::Request(_))
        }
    ));
    assert_eq!(pending.definition().as_bytes(), definition.as_bytes());
    assert_eq!(pending.advance_artifact().as_bytes(), original.as_bytes());
    finish(&mut pending, &execution);
    let seed = published(pending);
    let point = seed.source_point();
    assert_eq!(point.reference(), original.reference());
    assert_eq!(point.position(), 1);
    let mut bindings = program::Bindings { providers: 0 };
    let owner = seed
        .readmit_authored_owner(&mut bindings, &execution)
        .unwrap();
    assert_eq!(bindings.providers, 2);
    assert_eq!(
        owner.signal_owner().source_port().branches().unwrap().len(),
        1
    );
    assert_eq!(
        owner
            .signal_owner()
            .source_port()
            .observe(definition.branch())
            .unwrap(),
        *original.descriptor()
    );
    let calls = AtomicUsize::new(0);
    demand(&owner, value, &execution, &calls, expected.clone());
    demand(&owner, value, &execution, &calls, expected.clone());
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let recorded = owner
        .signal_owner()
        .read_definition_source(original.descriptor(), &execution)
        .unwrap();
    assert_eq!(recorded.as_bytes(), definition.as_bytes());
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        1
    );
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        3
    );
    drop((source, owner, observed, program.lowerings));
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
    assert_eq!(bindings.providers, 2);
    assert_eq!(
        owner
            .signal_owner()
            .read_definition_source(original.descriptor(), &execution)
            .unwrap()
            .as_bytes(),
        definition.as_bytes()
    );
    demand(&owner, value, &execution, &calls, expected);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn populated_initial_source_refuses_reinitialization_and_returns_the_actual_program() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut pending = prepare(program::program().builder, medium.clone(), &execution);
    finish(&mut pending, &execution);
    let definition = pending.definition().clone();
    drop(published(pending));
    let refusal = program::program()
        .builder
        .prepare_initial_authored_source(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            &execution,
        )
        .err()
        .expect("an existing original cannot be replaced by initialization");
    assert!(
        matches!(refusal.cause(), BridgeInitialSourceDenial::Signal(cause)
        if matches!(cause.as_ref(), SignalInitialSourceDenial::SourceNotEmpty { definitions: 1, advances: 1 }))
    );
    let (builder, _) = refusal.into_parts();
    let mut bindings = program::Bindings { providers: 0 };
    let owner = builder
        .readmit_authored_owner(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            &mut bindings,
            &execution,
        )
        .unwrap();
    assert_eq!(
        bindings.providers, 2,
        "the returned builder owns its original graph and lowerings"
    );
    assert_eq!(
        owner
            .signal_owner()
            .read_definition_source(
                &owner
                    .signal_owner()
                    .source_port()
                    .observe(definition.branch())
                    .unwrap(),
                &execution
            )
            .unwrap()
            .as_bytes(),
        definition.as_bytes()
    );
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn initial_section_refusal_returns_the_same_signal_program_without_publishing() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let (runtime, value, conditional) = owner_fixture::program();
    let count = runtime.graph().active_node_count();
    let refusal = runtime
        .prepare_initial_authored_source(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            owner_fixture::vocabulary(),
            &execution,
            |context| {
                assert!(context.graph().is_alive(value));
                assert_eq!(context.conditional_occurrences().len(), 1);
                assert_eq!(
                    context.conditional_occurrences()[0].node().index(),
                    conditional.index()
                );
                Err(SignalAuthoredInstallationRefused)
            },
        )
        .err()
        .expect("the actual section owner refused its declarations");
    assert!(matches!(
        refusal.cause(),
        SignalInitialSourceDenial::BridgeSectionRefused
    ));
    let (runtime, _) = refusal.into_parts();
    assert_eq!(runtime.graph().active_node_count(), count);
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        0
    );
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        0
    );
    drop(source);
    let mut pending = runtime
        .prepare_initial_authored_source(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            owner_fixture::vocabulary(),
            &execution,
            |_| {
                Ok(SignalBridgeAuthoredSection::from_bridge_bytes(
                    worth_execution::ExecutionRequestArrayBuilder::allocate(execution, 0)
                        .unwrap()
                        .seal()
                        .unwrap()
                        .into_immutable_bytes(),
                )
                .unwrap())
            },
        )
        .unwrap_or_else(|refusal| panic!("returned program cannot prepare: {:?}", refusal.cause()));
    let deadline = Instant::now() + Duration::from_secs(120);
    while pending.advance(&execution).unwrap() != SignalInitialSourceProgress::Published {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    let seed = pending
        .try_into_published()
        .unwrap_or_else(|_| panic!("actual initial ACK is required"));
    let owner = seed
        .readmit_authored_owner(&mut SignalOnlyOwnerInstallation, &execution)
        .unwrap();
    assert_eq!(owner.source_port().branches().unwrap().len(), 1);
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn initial_publication_refuses_programs_with_memory_branches_or_snapshots_before_section_capture() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    for snapshot in [false, true] {
        let (mut runtime, _, _) = owner_fixture::program();
        let basis = runtime
            .observe_signal_branch_basis(runtime.current_branch())
            .unwrap();
        if snapshot {
            runtime.capture_signal_branch_snapshot(&basis).unwrap();
        } else {
            runtime
                .fork_signal_branch("private-memory-branch", &basis)
                .unwrap();
        }
        drop(basis);
        let refusal = runtime
            .prepare_initial_authored_source(
                medium.clone(),
                SignalRecordedIdentityContribution::empty(media::owner()),
                owner_fixture::vocabulary(),
                &execution,
                |_| panic!("a non-initial program cannot reach section capture"),
            )
            .err()
            .expect("memory history cannot silently become an initial source");
        assert!(matches!(
            refusal.cause(),
            SignalInitialSourceDenial::ProgramNotInitial
        ));
        let (runtime, _) = refusal.into_parts();
        assert_eq!(runtime.known_branches().len(), if snapshot { 1 } else { 2 });
    }
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        0
    );
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        0
    );
    drop(source);
    media::close(serving, medium, &execution);
}
