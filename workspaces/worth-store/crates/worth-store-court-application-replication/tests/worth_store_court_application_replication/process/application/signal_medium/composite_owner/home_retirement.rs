//! The original stopped owner stays alive after its actual transport share retires.
use super::{fixture, media};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::*;

#[test]
fn stopped_signal_owner_retires_its_medium_before_original_allocations_are_reclaimed() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    assert_eq!(
        Arc::strong_count(&serving),
        2,
        "the actual Signal adapter owns one share"
    );
    owner.preflight_home_retirement(&execution).unwrap();
    let empty = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let release = ExecutionRequest::serial(&empty);
    owner.prepare_home_stop(&release).unwrap().commit();
    owner.commit_home_stop(&release).unwrap();
    let (candidate, _, _) = fixture::program();
    assert!(
        matches!(
            candidate.readmit_authored_owner(
                medium.clone(),
                SignalRecordedIdentityContribution::empty(media::owner()),
                fixture::vocabulary(),
                &mut SignalOnlyOwnerInstallation,
                &execution
            ),
            Err(SignalOwnerReadmissionDenial::Medium(
                SignalDurableMediumDenial::Unavailable
            ))
        ),
        "the actual stopped claim refuses readmission before its port is retired"
    );
    let original = owner
        .retain_stopped_home_allocation(&release)
        .unwrap_or_else(|(_, cause)| panic!("prepaid original retirement: {cause:?}"));
    assert_eq!(empty.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(
        Arc::strong_count(&serving),
        1,
        "the original remains alive without retaining physical service"
    );
    assert!(matches!(
        medium.capture_recovery_source(&execution),
        Err(SignalDurableMediumDenial::Unavailable)
    ));
    let (fresh, _, _) = fixture::program();
    assert!(matches!(
        fresh.readmit_authored_owner(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            fixture::vocabulary(),
            &mut SignalOnlyOwnerInstallation,
            &execution
        ),
        Err(SignalOwnerReadmissionDenial::Medium(
            SignalDurableMediumDenial::Unavailable
        ))
    ));
    media::close(serving, medium, &execution);
    // Physical Closed precedes destruction of the same owner and its supplier payloads.
    drop((original, reference));
}
