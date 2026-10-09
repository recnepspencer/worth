//! Real native competition at constructed-address and capability-open boundaries.

use super::*;
use crate::physical_runtime::ArtifactTreePathAllocationBoundary as Boundary;
use worth_store_physical_backend::ArtifactTreeDirectory;

#[test]
fn directory_address_denies_before_listing_and_retries_after_pressure_release() {
    deny_initial_path(Boundary::DirectoryAddress, 1);
}

#[test]
fn directory_open_denies_after_funded_address_before_listing_and_retries() {
    // Independent physical census of the actual empty-root -> WAL address.
    // No provider/open scratch formula is duplicated in the oracle.
    let directory = ArtifactTreeDirectory::families().child("wal").unwrap();
    let path_bytes = directory.owned_heap_bytes().unwrap();
    assert!(path_bytes > 0);
    deny_initial_path(Boundary::DirectoryOpen, path_bytes);
}

fn deny_initial_path(boundary: Boundary, path_allowance: u64) {
    let (directory, media, mut coordination) = world();
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let metadata = diagnostic_backing::storage_bytes() as u64;
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(ORIGINAL - metadata - path_allowance).unwrap(),
        )
        .unwrap();
    let mut discovery = media.bounded_discovery(8, 4096).unwrap();
    let failure = coordination
        .begin_source_read_allocation()
        .unwrap()
        .read_wal_payloads(&mut discovery, segments(1), ReadGrant::ceiling_only())
        .map_err(GrantedReadStop::unread)
        .unwrap_err();
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        offset,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::PathResidency {
                boundary: actual,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            },
    } = failure.diagnostic()
    else {
        panic!("exact constructed path/open denial required: {failure:?}");
    };
    assert_eq!(
        *actual, boundary,
        "the earlier address gate must not mask open pressure"
    );
    assert_eq!(offset, 0);
    assert!(requested > 0);
    assert_eq!(*admitted, ORIGINAL);
    match boundary {
        Boundary::DirectoryAddress => {
            assert!(requested as u64 > path_allowance);
            assert_eq!(*required, ORIGINAL - path_allowance + requested as u64);
        }
        Boundary::DirectoryOpen => assert_eq!(*required, ORIGINAL + requested as u64),
        _ => unreachable!("this fixture only targets initial directory work"),
    }
    assert_eq!(failure.charged_bytes(), 0);
    assert_eq!(discovery.counters().directory_entries_observed, 0);
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        held.bytes()
    );
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    drop(failure);
    drop(held);
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let observed = window
        .read_wal_payloads(&mut discovery, segments(1), ReadGrant::ceiling_only())
        .map_err(GrantedReadStop::unread)
        .unwrap();
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        observed.charged_bytes()
    );
    drop(observed);
    drop(window);
    drop(coordination);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    drop(ports);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

#[test]
fn file_address_denies_after_listing_and_context_then_same_owner_retries() {
    let (gate, schedule) = staged_pressure::listing_pause();
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let available = size_of::<ObservedWalArtifact>() as u64 + FIRST.len() as u64;
    let mut discovery = media.bounded_discovery(8, 4096).unwrap();
    let (result, held) = staged_pressure::read_under_pressure(
        &mut discovery,
        &mut coordination,
        &gate,
        1,
        available,
    );
    let failure = result.unwrap_err();
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalArtifact(name),
        offset,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::PathResidency {
                boundary: Boundary::FileAddress,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            },
    } = failure.diagnostic()
    else {
        panic!("file address must deny before file open or payload: {failure:?}");
    };
    assert_eq!(name, FIRST);
    assert_eq!(offset, 0);
    assert!(requested > 0);
    assert_eq!(*required, ORIGINAL + requested as u64);
    assert_eq!(*admitted, ORIGINAL);
    assert_eq!(discovery.counters().directory_entries_observed, 1);
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    let diagnostic = diagnostic_backing::storage_bytes() as u64 + FIRST.len() as u64;
    assert_eq!(failure.charged_bytes(), diagnostic);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        held.bytes() + diagnostic
    );
    let shared = failure.clone();
    drop(failure);
    drop(held);
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let observed = window
        .read_wal_payloads(&mut discovery, segments(1), ReadGrant::ceiling_only())
        .map_err(GrantedReadStop::unread)
        .unwrap();
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        observed.charged_bytes() + diagnostic
    );
    drop(observed);
    drop(window);
    drop(coordination);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        diagnostic
    );
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    drop(shared);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    drop(ports);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}
