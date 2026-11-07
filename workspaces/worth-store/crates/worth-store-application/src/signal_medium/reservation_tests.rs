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
    SignalPublicationSubject::new(SignalPublicationArtifact::Definition(
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
        .unwrap(),
    ))
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
