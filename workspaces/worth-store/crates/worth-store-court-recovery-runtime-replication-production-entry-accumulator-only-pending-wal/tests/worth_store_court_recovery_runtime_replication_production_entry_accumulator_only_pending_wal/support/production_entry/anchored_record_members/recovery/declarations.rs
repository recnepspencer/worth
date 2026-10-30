//! Anchored values with members survive abandoned runtimes through production
//! recovery, and a member whose durable bytes are damaged refuses with a
//! typed failure instead of reading as content or as absence.

pub(crate) use std::path::Path;

pub(crate) use worth_store::physical_runtime::AnchoredRecordMemberReadFailure;

pub(crate) use {crate::production_entry::anchored_record_members::assert_value, crate::production_entry::anchored_record_members::carried, crate::production_entry::anchored_record_members::content, crate::production_entry::anchored_record_members::new_member, crate::production_entry::anchored_record_members::publish_with_members};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::anchored_records::assert_journal, worth_store_testkit_recovery_runtime::production_entry::anchored_records::name, worth_store_testkit_recovery_runtime::production_entry::anchored_records::publish_journal_entry, worth_store_testkit_recovery_runtime::production_entry::anchored_records::value, worth_store_testkit_recovery_runtime::production_entry::anchored_records::CHECKPOINT};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::anchored_records};

/// Thirty members, inline and extent-backed, small enough that recovery
/// replays them from the WAL tail without a Store checkpoint.
pub(crate) fn first_contents() -> Vec<Vec<u8>> {
    (0..30_u32)
        .map(|index| match index {
            0 => content(index, 300_000),
            _ if index % 4 == 0 => content(index, 40_000 + index as usize),
            _ => content(index, 500 + index as usize * 311),
        })
        .collect()
}

/// Completes one fuzzy Store physical checkpoint.
pub(crate) fn store_checkpoint(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xa7; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(60_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("the checkpoint must start");
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "{outcome:?}"
    );
}

/// Flips one byte in the middle of every copy of `member` held by a record
/// artifact (not the WAL); returns the number of copies found.
pub(crate) fn flip_every_record_copy(root: &Path, member: &[u8]) -> usize {
    let probe = &member[member.len() / 2..member.len() / 2 + 64];
    let mut flipped = 0;
    let mut pending = vec![root.join("families").join("records")];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let mut bytes = std::fs::read(&path).unwrap();
            let Some(at) = bytes
                .windows(probe.len())
                .position(|window| window == probe)
            else {
                continue;
            };
            bytes[at + 32] ^= 0x5a;
            std::fs::write(&path, bytes).unwrap();
            flipped += 1;
        }
    }
    flipped
}
