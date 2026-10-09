//! Payload admission, replacement, and transfer retain one ledger charge.
use super::{authority, request, MemoryLimitDenial, MemoryLimitLevel, PROCESS_BYTES, SERIAL};
use std::cell::Cell;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial};

#[test]
fn the_authority_reports_its_configuration() {
    let config = authority().config();
    assert_eq!(config.max_workers.get(), 2);
    assert_eq!(config.charged_memory_bytes, Some(PROCESS_BYTES));
}

// Declaration order frees the backing before releasing its admitted charge.
struct Payload {
    backing: Vec<u8>,
    ticket: ExecutionMemoryReservation,
}

fn payload(
    lease: &ExecutionResourceLease<'_>,
    bytes: usize,
    allocations: &Cell<usize>,
) -> Result<Payload, MemoryLimitDenial> {
    let ticket = lease.reserve_memory(u64::try_from(bytes).unwrap())?;
    // No payload allocation is attempted until the actual ledger admits it.
    allocations.set(allocations.get() + 1);
    let mut backing = Vec::new();
    backing.try_reserve_exact(bytes).unwrap();
    assert_eq!(
        backing.capacity(),
        bytes,
        "check actual admitted Vec backing"
    );
    backing.resize(bytes, 7);
    Ok(Payload { backing, ticket })
}

#[test]
fn payload_admission_precedes_growth_and_charges_process_and_ancestors() {
    let _serial = SERIAL.lock().unwrap();
    let authority = authority();
    let parent = authority.request_lease(request(128)).unwrap();
    let first = parent.child(request(96)).unwrap();
    let sibling = parent.child(request(96)).unwrap();
    let allocations = Cell::new(0);
    let held = payload(&first, 64, &allocations).unwrap();
    assert_eq!(held.ticket.bytes(), 64);
    assert!(matches!(
        payload(&sibling, 65, &allocations),
        Err(MemoryLimitDenial {
            requested: 65,
            admitted: 64,
            level: MemoryLimitLevel::Policy { ancestor: 1 }
        })
    ));
    assert_eq!(allocations.get(), 1, "refusal precedes payload allocation");
    assert_eq!(held.backing, vec![7; 64], "existing backing remains intact");
    let other = payload(&sibling, 64, &allocations).unwrap();
    assert!(matches!(
        parent.reserve_memory(1),
        Err(MemoryLimitDenial {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));

    // Each unrelated lease can admit its own request; their shared process cannot.
    let unrelated = authority
        .request_lease(request(PROCESS_BYTES - 1_024))
        .unwrap();
    let occupied = unrelated.reserve_memory(PROCESS_BYTES - 1_024).unwrap();
    let process_probe = authority.request_lease(request(1_024)).unwrap();
    assert!(matches!(
        process_probe.reserve_memory(897),
        Err(MemoryLimitDenial {
            requested: 897,
            admitted: 896,
            level: MemoryLimitLevel::Process
        })
    ));
    drop(occupied);
    assert_eq!(process_probe.reserve_memory(897).unwrap().bytes(), 897);
    drop(held);
    drop(other);
    let all = authority.request_lease(request(PROCESS_BYTES)).unwrap();
    assert_eq!(
        all.reserve_memory(PROCESS_BYTES).unwrap().bytes(),
        PROCESS_BYTES
    );
}

#[test]
fn replacement_backings_coexist_and_custody_outlives_the_lease() {
    let _serial = SERIAL.lock().unwrap();
    let authority = authority();
    let allocations = Cell::new(0);
    let retained = {
        let lease = authority.request_lease(request(128)).unwrap();
        let old = payload(&lease, 48, &allocations).unwrap();
        let replacement = payload(&lease, 80, &allocations).unwrap();
        assert_eq!(old.backing.len(), 48);
        assert_eq!(replacement.backing.len(), 80);
        assert!(matches!(
            lease.reserve_memory(1),
            Err(MemoryLimitDenial {
                requested: 1,
                admitted: 0,
                level: MemoryLimitLevel::Policy { ancestor: 0 }
            })
        ));
        drop(old); // Old backing is freed before its ticket.
        assert_eq!(lease.reserve_memory(48).unwrap().bytes(), 48);
        replacement
    };
    let process_probe = authority.request_lease(request(PROCESS_BYTES)).unwrap();
    assert!(matches!(
        process_probe.reserve_memory(PROCESS_BYTES - 79),
        Err(MemoryLimitDenial { requested, admitted, level: MemoryLimitLevel::Process }) if requested == PROCESS_BYTES - 79 && admitted == PROCESS_BYTES - 80
    ));
    assert_eq!(retained.backing, vec![7; 80]);
    drop(retained);
    assert_eq!(
        process_probe.reserve_memory(PROCESS_BYTES).unwrap().bytes(),
        PROCESS_BYTES
    );
}

#[test]
fn same_byte_transfer_is_atomic_and_releases_only_the_old_lineage() {
    let _serial = SERIAL.lock().unwrap();
    let authority = authority();
    let source = authority.request_lease(request(64)).unwrap();
    let target = authority.request_lease(request(64)).unwrap();
    let allocations = Cell::new(0);
    let mut retained = payload(&source, 64, &allocations).unwrap();
    let blocker = target.reserve_memory(1).unwrap();
    assert_eq!(
        target.transfer_memory(&mut retained.ticket),
        Err(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            requested: 64,
            admitted: 63,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        }))
    );
    assert_eq!(retained.ticket.bytes(), 64);
    assert!(matches!(
        source.reserve_memory(1),
        Err(MemoryLimitDenial {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    assert_eq!(retained.backing, vec![7; 64]);
    drop(blocker);
    target.transfer_memory(&mut retained.ticket).unwrap();
    assert_eq!(retained.ticket.bytes(), 64);
    assert_eq!(source.reserve_memory(64).unwrap().bytes(), 64);
    assert!(matches!(
        target.reserve_memory(1),
        Err(MemoryLimitDenial {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    assert_eq!(
        allocations.get(),
        1,
        "transfer does not allocate replacement backing"
    );
    drop(retained);
    assert_eq!(target.reserve_memory(64).unwrap().bytes(), 64);
}
