//! Reservation is custody even before a physical endpoint exists.
use super::publication_task::SignalPublications;
use super::subject::{SignalPublicationArtifact, SignalPublicationSubject};
use std::sync::Arc;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalOwnerIdentity, HomeIdentity,
    SignalDefinitionTransitionIdentity, SignalOwnerSlotIdentity,
};
use worth_signal::facade::durability::{
    SignalBridgeAuthoredSection, SignalDefinitionTransitionArtifact,
    SignalDurableMediumDenial as Denial, SignalPublicationCompletion, SignalPublicationObservation,
    SignalPublicationTaskPosture,
};

fn subject(execution: &ExecutionRequest<'_, '_>, ordinal: u64) -> SignalPublicationSubject {
    let owner = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(2),
    );
    SignalPublicationSubject::new(SignalPublicationArtifact::Definition(definition(
        execution, ordinal, owner,
    )))
}

fn definition(
    execution: &ExecutionRequest<'_, '_>,
    ordinal: u64,
    owner: DurableSignalOwnerIdentity,
) -> SignalDefinitionTransitionArtifact {
    let graph = Box::new(worth_signal::facade::SignalGraph::new());
    let branch = DurableSignalBranchIdentity::from_recorded(owner, 3);
    let image = graph.capture_authored_image(execution, branch).unwrap();
    let vocabulary =
        worth_signal::facade::durability::SignalDurableTierVocabulary::declare([((), "base")])
            .unwrap();
    let runtime = worth_signal::facade::runtime::RuntimeConfig::<()>::new()
        .capture_authored_image(&graph, &vocabulary, branch, execution)
        .unwrap();
    let bytes = ExecutionRequestArrayBuilder::allocate(*execution, 0).unwrap();
    let bridge = SignalBridgeAuthoredSection::from_bridge_bytes(
        bytes.seal().unwrap().into_immutable_bytes(),
    )
    .unwrap();
    SignalDefinitionTransitionArtifact::capture(
        execution,
        SignalDefinitionTransitionIdentity::from_recorded(owner, ordinal),
        branch,
        None,
        &image,
        &runtime,
        &[],
        &bridge,
    )
    .unwrap()
}

#[test]
fn reserved_publication_cannot_be_replaced_before_a_driver_or_physical_source_exists() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let original = subject(&execution, 1);
    let successor = subject(&execution, 2);
    let publications = SignalPublications::default();
    let task = publications.task(&original, &execution).unwrap();
    assert_eq!(
        publications.posture(),
        SignalPublicationTaskPosture::Unsettled
    );
    assert!(matches!(
        publications.task(&successor, &execution),
        Err(Denial::PublicationBusy)
    ));
    assert!(Arc::ptr_eq(
        &task,
        &publications.task(&original, &execution).unwrap()
    ));
    let driver = task.try_driver().unwrap();
    assert_eq!(publications.posture(), SignalPublicationTaskPosture::Busy);
    assert!(matches!(
        publications.task(&successor, &execution),
        Err(Denial::PublicationBusy)
    ));
    let _ = task.remember_refusal(SignalPublicationObservation::Finished(
        SignalPublicationCompletion::proven_no_effect(Denial::Unavailable),
    ));
    assert_eq!(publications.posture(), SignalPublicationTaskPosture::Busy);
    drop(driver);
    assert_eq!(
        publications.posture(),
        SignalPublicationTaskPosture::Terminal
    );
    let next = publications.task(&successor, &execution).unwrap();
    assert!(!Arc::ptr_eq(&task, &next));
}

#[test]
fn indeterminate_start_retains_original_custody_and_refuses_replacement() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let original = subject(&execution, 1);
    let successor = subject(&execution, 2);
    let publications = SignalPublications::default();
    let task = publications.task(&original, &execution).unwrap();
    let _ = task.remember_refusal(SignalPublicationObservation::Finished(
        SignalPublicationCompletion::indeterminate(Denial::PublicationUnresolved),
    ));
    assert_eq!(
        publications.posture(),
        SignalPublicationTaskPosture::Unsettled
    );
    assert!(matches!(
        publications.task(&successor, &execution),
        Err(Denial::PublicationUnresolved)
    ));
    assert!(Arc::ptr_eq(
        &task,
        &publications.task(&original, &execution).unwrap()
    ));
}

#[test]
fn retry_driver_preserves_unresolved_original_and_registry_reservation() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let original = subject(&execution, 1);
    let successor = subject(&execution, 2);
    let publications = SignalPublications::default();
    let task = publications.task(&original, &execution).unwrap();
    let driver = task.try_driver().unwrap();
    let _ = task.remember_refusal(SignalPublicationObservation::Finished(
        SignalPublicationCompletion::indeterminate(Denial::PublicationUnresolved),
    ));
    driver.retry_no_effect();
    drop(driver);
    assert_eq!(
        publications.posture(),
        SignalPublicationTaskPosture::Unsettled
    );
    assert!(
        matches!(task.observation(), Some(SignalPublicationObservation::Finished(done))
        if matches!(done.fate(), worth_foundational::facade::DurablePublicationFate::Indeterminate { .. }))
    );
    assert!(matches!(
        publications.task(&successor, &execution),
        Err(Denial::PublicationUnresolved)
    ));
    assert!(Arc::ptr_eq(
        &task,
        &publications.task(&original, &execution).unwrap()
    ));
}

#[test]
fn contended_signal_original_keeps_its_cancellation_when_advanced_by_a_live_receiver() {
    use worth_signal::facade::durability::SignalDurableMedium;
    use worth_store::physical_runtime::*;
    let directory = tempfile::tempdir().unwrap();
    let live = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&live);
    let home = crate::tests::lifecycle::open(directory.path(), &request);
    let medium = home
        .context
        .world_slot
        .as_ref()
        .unwrap()
        .signal_slot()
        .attach(None, None, &request)
        .unwrap();
    let first = home
        .context
        .serving
        .ordered_journal(OrderedJournalName::new("query.corrections").unwrap())
        .unwrap();
    let view = request
        .run_native_with_checkpoint(|context| context.retain_interruption_view(None))
        .unwrap()
        .0
        .unwrap();
    let input = OrderedJournalPublication::new(
        1,
        b"first-original".to_vec(),
        crate::open::physical_admission::placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let OrderedJournalPublicationStart::Ready {
        operation,
        recovery,
    } = first.start_publication(input, view)
    else {
        panic!("first admission");
    };
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let artifact = definition(&request, 1, medium.binding().owner());
    assert!(matches!(
        medium.advance_definition(&artifact, &ExecutionRequest::serial(&original)),
        SignalPublicationObservation::Pending
    ));
    assert_eq!(
        medium.publication_task_posture(),
        SignalPublicationTaskPosture::Unsettled
    );
    cancellation.cancel();
    let observed = medium.advance_definition(&artifact, &request);
    assert!(
        matches!(observed, SignalPublicationObservation::Finished(done) if matches!(done.fate(), worth_foundational::facade::DurablePublicationFate::ProvenNoEffect { .. }))
    );
    drop(operation);
    drop(recovery);
    // Admission is now free. The same cancelled original remains terminal.
    assert!(
        matches!(medium.advance_definition(&artifact, &request), SignalPublicationObservation::Finished(done) if matches!(done.fate(), worth_foundational::facade::DurablePublicationFate::ProvenNoEffect { .. }))
    );
    assert_eq!(
        medium.publication_task_posture(),
        SignalPublicationTaskPosture::Terminal
    );
    let journal = home
        .context
        .serving
        .ordered_journal(OrderedJournalName::new("signal.definitions").unwrap())
        .unwrap();
    assert!(journal.head().unwrap().is_none());
}
