//! Actual companion epochs, issued registrations and deferred participant Drop.
use super::*;
use crate::mvcc::publication::companion::{
    CompanionPreflightStop, PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion,
};
use crate::tests::support::runtime_with_test_schema;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[derive(Debug)]
struct Participant {
    drops: Arc<AtomicUsize>,
    prepares: Arc<AtomicUsize>,
}
impl Drop for Participant {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl RelationalPublicationCompanion for Participant {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.prepares.fetch_add(1, Ordering::SeqCst);
        let cell = context.mint_selected_branch_cell(Arc::new(0_u64))?;
        let reserved = cell.reserve_preflight(context)?;
        context.seal_replacement(reserved, Arc::new(1_u64))
    }
}

fn install(
    port: &PublicationCompanionRegistrationPort,
) -> (
    PublicationCompanionRegistration,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
) {
    let drops = Arc::new(AtomicUsize::new(0));
    let prepares = Arc::new(AtomicUsize::new(0));
    let registration = port
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(Participant {
            drops: Arc::clone(&drops),
            prepares: Arc::clone(&prepares),
        }))
        .unwrap();
    (registration, drops, prepares)
}

#[test]
fn original_epoch_refusal_retries_without_callback_or_participant_destruction() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let port = runtime.publication_companion_port();
    let (registration, drops, prepares) = install(&port);
    let generation = registration.generation;
    let client = Arc::as_ptr(&registration._client);
    let epoch = port.publication.companion_registry().enter().unwrap();
    assert_eq!(epoch.active().unwrap().unwrap().0, generation);
    let (registration, denial) = port
        .try_retire_required_for_request(registration, &request)
        .unwrap_err();
    assert_eq!(
        denial,
        PublicationCompanionRetirementDenial::Registration(
            PublicationCompanionRegistrationStop::PublicationPending
        )
    );
    assert_eq!(Arc::as_ptr(&registration._client), client);
    assert_eq!(epoch.active().unwrap().unwrap().0, generation);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(epoch);
    let retired = port
        .try_retire_required_for_request(registration, &request)
        .unwrap();
    let epoch = port.publication.companion_registry().enter().unwrap();
    assert!(matches!(
        epoch.active(),
        Err(PublicationCompanionRegistrationStop::RebindRequired)
    ));
    assert!(
        matches!(&*epoch.state, CompanionRegistrationState::RequiredRebind { generation: actual } if *actual == generation)
    );
    assert!(!epoch.has_live_close_client());
    drop(epoch);
    assert_eq!(prepares.load(Ordering::SeqCst), 0);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    retired.reclaim();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn stopped_request_preserves_same_issued_registration_and_delivery_posture() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let port = runtime.publication_companion_port();
    let (registration, drops, prepares) = install(&port);
    let generation = registration.generation;
    let client = Arc::as_ptr(&registration._client);
    stop.cancel();
    let (registration, denial) = port
        .try_retire_required_for_request(registration, &request)
        .unwrap_err();
    assert!(matches!(
        denial,
        PublicationCompanionRetirementDenial::Execution(_)
    ));
    assert_eq!(registration.generation, generation);
    assert_eq!(Arc::as_ptr(&registration._client), client);
    let epoch = port.publication.companion_registry().enter().unwrap();
    assert_eq!(epoch.active().unwrap().unwrap().0, generation);
    drop(epoch);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(prepares.load(Ordering::SeqCst), 0);
    port.remove_required(&registration).unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn foreign_and_superseded_registrations_leave_the_actual_successor_installed() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let other_runtime = runtime_with_test_schema();
    let port = runtime.publication_companion_port();
    let other_port = other_runtime.publication_companion_port();
    let (registration, drops, _) = install(&port);
    let generation = registration.generation;
    let client = Arc::as_ptr(&registration._client);
    let (registration, denial) = other_port
        .try_retire_required_for_request(registration, &request)
        .unwrap_err();
    assert_eq!(
        denial,
        PublicationCompanionRetirementDenial::Registration(
            PublicationCompanionRegistrationStop::Superseded
        )
    );
    assert_eq!(Arc::as_ptr(&registration._client), client);
    assert_eq!(
        port.publication
            .companion_registry()
            .enter()
            .unwrap()
            .active()
            .unwrap()
            .unwrap()
            .0,
        generation
    );
    let (successor, successor_drops, _) = install(&port);
    assert!(successor.generation > generation);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let (registration, denial) = port
        .try_retire_required_for_request(registration, &request)
        .unwrap_err();
    assert_eq!(
        denial,
        PublicationCompanionRetirementDenial::Registration(
            PublicationCompanionRegistrationStop::Superseded
        )
    );
    assert_eq!(Arc::as_ptr(&registration._client), client);
    assert_eq!(
        port.publication
            .companion_registry()
            .enter()
            .unwrap()
            .active()
            .unwrap()
            .unwrap()
            .0,
        successor.generation
    );
    assert_eq!(successor_drops.load(Ordering::SeqCst), 0);
    drop(registration);
    port.try_retire_required_for_request(successor, &request)
        .unwrap()
        .reclaim();
    assert_eq!(successor_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn exact_cleanup_after_owner_stop_retains_original_payload_until_reclaim() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let port = runtime.publication_companion_port();
    let (registration, drops, prepares) = install(&port);
    let generation = registration.generation;
    drop(runtime);
    assert!(!port.owner.accepts_operations());
    let retired = port
        .try_retire_required_for_request(registration, &request)
        .unwrap();
    let epoch = port.publication.companion_registry().enter().unwrap();
    assert!(
        matches!(&*epoch.state, CompanionRegistrationState::RequiredRebind { generation: actual } if *actual == generation)
    );
    assert!(matches!(
        epoch.active(),
        Err(PublicationCompanionRegistrationStop::RebindRequired)
    ));
    drop(epoch);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(prepares.load(Ordering::SeqCst), 0);
    drop(port);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    retired.reclaim();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
