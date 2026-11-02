//! Authentic closed source custody survives loss of an authenticated local handoff.
use super::super::{
    authenticated_owner::BankHttpAuthenticatedOwner,
    mutation_command::{admit, MutationCommand},
    mutation_custody::{MutationRecord, SourceMutationRecord},
    mutation_executor::WorkerCommand,
};
use super::fixture::application;
use super::mutation::{command, create, decode};
use bank_domain::model::AccountId;
use bank_server::BankMutationControls;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot, Semaphore};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[tokio::test]
async fn lost_closed_source_handoff_restores_the_actual_owner_and_its_capacity() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = application.authenticate(&scope).await;
    let request = application.runtime.request(&principal, &scope);
    let create = admit(decode(create("closed-handoff", 17)), Duration::from_secs(5)).unwrap();
    let MutationCommand::Create {
        input,
        key,
        history,
    } = create.command
    else {
        panic!("actual create command")
    };
    let (source, refusal) = super::super::mutation_construction::start(
        &application.runtime,
        &principal,
        input,
        BankMutationControls::new(scope.clone(), key),
        &request,
        history,
        None,
    )
    .unwrap();
    assert!(refusal.is_none());
    let closed = source
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("actual source closes: {cause:?}"));
    let permits = Arc::new(Semaphore::new(1));
    let record = super::super::closed_mutation_custody::ClosedMutationRecord::from_source(
        SourceMutationRecord {
            owner: BankHttpAuthenticatedOwner::from_principal(&principal),
            sequence: 2,
            custody: closed,
            released_cleanup: None,
            _permit: Arc::clone(&permits).try_acquire_owned().unwrap(),
        },
    )
    .unwrap_or_else(|_| panic!("the actual source is physically closed"));
    let retired = record.evidence.closed().retired_component_count();
    let mut records =
        HashMap::from([("closed-handoff".to_owned(), MutationRecord::Closed(record))]);
    let (returns, mut returned) = mpsc::channel(4);
    // A stale sequence cannot move the native owner or release its capacity.
    let (response, receiver) = oneshot::channel();
    super::super::closed_mutation_handoff::execute(
        Arc::clone(&application),
        &mut records,
        admit(
            decode(command("closed-handoff", "close", 2)),
            Duration::from_secs(5),
        )
        .unwrap(),
        cancellation.clone(),
        response,
        returns.clone(),
    )
    .await;
    assert!(receiver.await.unwrap().is_err());
    assert_eq!(permits.available_permits(), 0);
    assert!(matches!(
        records.get("closed-handoff"),
        Some(MutationRecord::Closed(_))
    ));
    // Losing the response returns the exact record, not a recreated receipt.
    let (response, receiver) = oneshot::channel();
    super::super::closed_mutation_handoff::execute(
        Arc::clone(&application),
        &mut records,
        admit(
            decode(command("closed-handoff", "close", 3)),
            Duration::from_secs(5),
        )
        .unwrap(),
        cancellation.clone(),
        response,
        returns.clone(),
    )
    .await;
    assert!(matches!(
        records.get("closed-handoff"),
        Some(MutationRecord::ClosedHandoff)
    ));
    drop(receiver);
    let WorkerCommand::ReturnClosedHandoff { id, record } = returned.recv().await.unwrap() else {
        panic!("lost reply returns original custody")
    };
    assert_eq!(permits.available_permits(), 0);
    assert!(matches!(
        records.remove(&id),
        Some(MutationRecord::ClosedHandoff)
    ));
    records.insert(id, MutationRecord::Closed(record));
    // The same fresh sequence can receive the genuine owner after restoration.
    let (response, receiver) = oneshot::channel();
    super::super::closed_mutation_handoff::execute(
        Arc::clone(&application),
        &mut records,
        admit(
            decode(command("closed-handoff", "close", 3)),
            Duration::from_secs(5),
        )
        .unwrap(),
        cancellation,
        response,
        returns,
    )
    .await;
    let delivery = receiver
        .await
        .unwrap()
        .unwrap_or_else(|cause| panic!("original handoff: {cause:?}"));
    let handoff = delivery.receive();
    assert_eq!(
        handoff.evidence().closed().retired_component_count(),
        retired
    );
    assert!(std::ptr::eq(handoff.runtime(), &application.runtime));
    assert_eq!(
        permits.available_permits(),
        1,
        "confirmed local delivery releases transport capacity"
    );
    let WorkerCommand::ConfirmClosedHandoff { id } = returned.recv().await.unwrap() else {
        panic!("confirmed receipt removes the worker marker")
    };
    assert!(matches!(
        records.remove(&id),
        Some(MutationRecord::ClosedHandoff)
    ));
    assert!(records.is_empty());
    let (_runtime, evidence) = handoff.into_parts();
    assert_eq!(evidence.closed().retired_component_count(), retired);
}

mod retained_effect;
