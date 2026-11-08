//! One physical serving allocation can admit only one live canonical Signal owner.
use super::{medium_fixture as media, owner_fixture as fixture};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_store::physical_runtime::AdmittedRecordPlacementPolicy;
use worth_store_application::StoreSignalMedium;

fn second_admission(
    medium: Arc<dyn SignalDurableMedium>,
    execution: &ExecutionRequest<'_, '_>,
) -> SignalLiveOwnerIdentity {
    match fixture::program().0.readmit_authored_owner(
        medium,
        SignalRecordedIdentityContribution::empty(media::owner()),
        fixture::vocabulary(),
        &mut worth_signal::facade::durability::SignalOnlyOwnerInstallation,
        execution,
    ) {
        Err(SignalOwnerReadmissionDenial::Medium(
            SignalDurableMediumDenial::OwnerAlreadyReadmitted(holder),
        )) => holder,
        Err(other) => panic!("second admission must name the actual live owner: {other:?}"),
        Ok(_owner) => panic!("a shared medium cannot mint a competing canonical owner"),
    }
}

#[test]
fn second_live_owner_refuses_before_publication_and_original_owner_can_continue() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let source = medium.capture_recovery_source(&execution).unwrap();
    let before = source
        .frontier(SignalDurableJournal::Advances, &execution)
        .unwrap();
    drop(source);
    let holder = second_admission(medium.clone(), &execution);
    assert_eq!(holder.owner(), owner.binding().owner());
    assert_eq!(holder.incarnation(), owner.binding().incarnation());
    assert_eq!(second_admission(medium.clone(), &execution), holder);
    assert!(matches!(medium.retire(&execution),
        Err(SignalDurableMediumDenial::OwnerAlreadyReadmitted(actual)) if actual == holder));
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        before
    );
    drop(source);
    let (&branch, original) = reference.heads.first_key_value().unwrap();
    assert_eq!(&owner.source_port().observe(branch).unwrap(), original);
    let handle = owner.source_port().readmit_descriptor(original).unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let task = owner
        .mutation_port()
        .begin_advance(execution, &handle, &mut (), |_| {
            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
        .unwrap();
    let output = fixture::finish(task, &execution);
    let committed = output.artifact().descriptor().clone();
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    drop((output, handle, owner));
    let owner = fixture::readmit(medium.clone(), &execution);
    assert_eq!(owner.source_port().observe(branch).unwrap(), committed);
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn failed_readmission_releases_its_original_exclusive_claim_before_retry() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    fixture::seed(medium.as_ref(), &execution);
    let foreign = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([9; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    );
    let refused = fixture::program().0.readmit_authored_owner(
        medium.clone(),
        SignalRecordedIdentityContribution::empty(foreign),
        fixture::vocabulary(),
        &mut worth_signal::facade::durability::SignalOnlyOwnerInstallation,
        &execution,
    );
    assert!(matches!(
        refused,
        Err(SignalOwnerReadmissionDenial::ForeignOwner)
    ));
    let owner = fixture::readmit(medium.clone(), &execution);
    assert_eq!(owner.source_port().branches().unwrap().len(), 3);
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn alternate_adapters_require_released_unshared_serving_and_attachment_refusals_retain_it() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let serving = match Arc::try_unwrap(serving) {
        Err(shared) => shared,
        Ok(_original) => panic!("a live medium must prevent a second owned attachment input"),
    };
    assert!(matches!(
        medium.retire(&execution),
        Err(SignalDurableMediumDenial::OwnerAlreadyReadmitted(_))
    ));
    let binding = medium.binding().clone();
    drop(owner);
    medium.retire(&execution).unwrap();
    let serving = match Arc::try_unwrap(serving) {
        Ok(original) => Box::new(original),
        Err(shared) => panic!(
            "retirement retains {} physical shares",
            Arc::strong_count(&shared)
        ),
    };
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    let placement: AdmittedRecordPlacementPolicy =
        worth_store_testkit_store::physical_record_journeys::configuration().1;
    let refused = match StoreSignalMedium::from_serving(
        serving,
        binding.clone(),
        placement,
        &ExecutionRequest::serial(&stopped),
    ) {
        Err(refused) => refused,
        Ok(_attached) => panic!("stopped attachment must retain its original unshared runtime"),
    };
    assert!(matches!(
        refused.cause(),
        SignalDurableMediumDenial::Request(_)
    ));
    let (serving, replacement) = refused.try_attach(&execution).unwrap();
    // The old registry stays retired, even when its public Arc is retained.
    let old = fixture::program().0.readmit_authored_owner(
        medium.clone(),
        SignalRecordedIdentityContribution::empty(media::owner()),
        fixture::vocabulary(),
        &mut worth_signal::facade::durability::SignalOnlyOwnerInstallation,
        &execution,
    );
    assert!(matches!(
        old,
        Err(SignalOwnerReadmissionDenial::Medium(
            SignalDurableMediumDenial::Unavailable
        ))
    ));
    let owner = fixture::readmit(replacement.clone(), &execution);
    assert_eq!(owner.source_port().branches().unwrap().len(), 3);
    drop(owner);
    media::close(serving, replacement, &execution);
}
