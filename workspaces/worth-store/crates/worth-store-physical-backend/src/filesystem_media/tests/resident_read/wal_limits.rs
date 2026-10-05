//! A WAL inventory's refusals name the bound they passed and both counts:
//! the listing's segment ceiling, and the byte ceiling its caller named for
//! the whole inventory, which holds what earlier reads in it took.

use super::*;
use FilesystemObservationBound::{Entries, RequestedBytes};

fn wal_discovery(files: &[&str]) -> (tempfile::TempDir, BoundedRecoveryFilesystemDiscovery) {
    let (parent, _observer, discovery) = discovery(
        |root| {
            let wal = root.join("families/wal");
            std::fs::create_dir_all(&wal).expect("real WAL directory");
            for name in files {
                std::fs::write(wal.join(name), b"12345").expect("real WAL file");
            }
        },
        8,
        64,
    );
    (parent, discovery)
}

/// The caller-managed and the allocated inventory's refusals, named.
fn inventory_refusals(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    maximum_segments: u64,
    byte_limit: u64,
) -> [Option<(FilesystemObservationBound, u64, u64)>; 2] {
    let maximum_segments = segments(maximum_segments);
    let managed = discovery.read_wal_artifacts(maximum_segments, byte_limit);
    let allocated = discovery.read_wal_artifacts_with_allocators(
        maximum_segments,
        byte_limit,
        |count| Ok::<_, DeniedAllocation>(Vec::with_capacity(count)),
        |length| Ok(vec![0; length]),
        |count| Ok(std::ffi::OsString::with_capacity(count)),
    );
    [
        managed.err().as_ref().and_then(named),
        match allocated {
            Err(RecoveryDiscoveryAllocationFailure::Discovery(failure)) => named(&failure),
            _ => None,
        },
    ]
}

#[test]
fn a_listing_past_its_segment_ceiling_names_both_counts() {
    let (_parent, mut discovery) = wal_discovery(&["a.wal", "b.wal", "c.wal"]);
    assert_eq!(
        inventory_refusals(&mut discovery, 2, 64),
        [Some((Entries, 3, 2)); 2]
    );
    assert_eq!(discovery.counters().bytes_read, 0);
}

/// Two five-byte files under an inventory ceiling of eight: the second read
/// is asked for the three bytes left, and the refusal counts the five the
/// first read held.
#[test]
fn an_inventory_past_its_byte_ceiling_counts_what_earlier_reads_held() {
    let (_parent, mut discovery) = wal_discovery(&["a.wal", "b.wal"]);
    assert_eq!(
        inventory_refusals(&mut discovery, 2, 8),
        [Some((RequestedBytes, 10, 8)); 2]
    );
}
