use std::io::Cursor;

use super::checkpoint_backup_verification::{
    verify_bounded_checkpoint_backup_artifact_from_reader, BoundedCheckpointBackupDenial,
};

#[path = "checkpoint_backup_fixture.rs"]
mod checkpoint_backup_fixture;
use checkpoint_backup_fixture::{
    raw_fixture, raw_fixture_with_pages, rehashed_byte_fixture, rehashed_page_swap,
    rehashed_u64_fixture, request,
};

#[test]
fn independent_wire_fixture_exposes_exact_frontier_and_budget_observations() {
    let fixture = raw_fixture(1, 10);
    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let observation = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, 256),
    )
    .expect("independent checkpoint wire fixture");

    assert_eq!(observation.root_reference(), 1);
    assert_eq!(observation.root_generation(), 1);
    assert_eq!(observation.covered_lsn(), (1, 11));
    assert_eq!(observation.redo_lsn(), 10);
    assert_eq!(observation.page_count(), 1);
    assert_eq!(observation.bytes_read(), fixture.bytes.len() as u64);
    assert_eq!(
        observation.decoder_allocation_bytes(),
        fixture.decoder_allocation_bytes()
    );
    assert_eq!(observation.peak_buffer_bytes(), fixture.peak_buffer_bytes());
    assert_eq!(observation.artifact_digest(), fixture.digest);
}

#[test]
fn same_generation_wrong_root_is_denied_after_raw_bytes_are_rehashed() {
    let fixture = raw_fixture(2, 10);
    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, 256),
    )
    .expect_err("owner root binding must reject a different root");

    assert!(matches!(
        denial,
        BoundedCheckpointBackupDenial::BindingMismatch
    ));
}

#[test]
fn rehashed_stale_page_frontier_is_denied_by_the_format_owner() {
    let fixture = raw_fixture(1, 9);
    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, 256),
    )
    .expect_err("page LSN below redo boundary");

    assert!(matches!(
        denial,
        BoundedCheckpointBackupDenial::InvalidPageFrontier
    ));
}

#[test]
fn rehashed_frontier_mutations_are_denied_by_the_owner_binding() {
    let original = raw_fixture(1, 10);
    let mutations = [
        (42, 2, "covered-LSN start"),
        (50, 12, "covered-LSN end"),
        (58, 9, "redo LSN"),
        (34, 2, "root generation"),
        (78 + 8, 2, "page identity"),
        (78 + 16, 2, "page generation"),
        (78 + 24, 11, "page LSN"),
    ];

    for (offset, value, field) in mutations {
        let fixture = rehashed_u64_fixture(&original, offset, value);
        let mut reader = Cursor::new(fixture.bytes.as_slice());
        let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
            &mut reader,
            fixture.bytes.len() as u64,
            request(&fixture, 256),
        )
        .expect_err(field);
        assert!(
            matches!(&denial, BoundedCheckpointBackupDenial::BindingMismatch),
            "{field} mutation must not redefine the owner frontier: {denial:?}"
        );
    }
}

#[test]
fn rehashed_checkpoint_identity_mutation_is_denied_by_the_owner_binding() {
    let original = raw_fixture(1, 10);
    let identity_offset = 78 + 32;
    let fixture = rehashed_byte_fixture(&original, identity_offset, b'X');
    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, 256),
    )
    .expect_err("checkpoint identity mutation");
    assert!(matches!(
        denial,
        BoundedCheckpointBackupDenial::BindingMismatch
    ));
}

#[test]
fn persisted_frontier_rejects_authority_substitution_without_artifact_mutation() {
    let fixture = raw_fixture(1, 10);
    let mut substituted = request(&fixture, 256);
    substituted.expected_authority_fingerprint = [8; 32];

    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        substituted,
    )
    .expect_err("a foreign authority fingerprint must not reinterpret persisted bytes");

    assert!(matches!(
        denial,
        BoundedCheckpointBackupDenial::BindingMismatch
    ));
}

#[test]
fn rehashed_page_count_and_order_mutations_are_denied() {
    let original = raw_fixture_with_pages(1, &[10, 10]);
    let count_mutation = rehashed_u64_fixture(&original, 66, 1);
    let mut reader = Cursor::new(count_mutation.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        count_mutation.bytes.len() as u64,
        request(&count_mutation, 512),
    )
    .expect_err("page count mutation");
    assert!(
        !matches!(
            denial,
            BoundedCheckpointBackupDenial::InternalDigestMismatch
        ),
        "the rehashed page-count mutation must reach semantic validation: {denial:?}"
    );

    let reordered = rehashed_page_swap(&original);
    let mut reader = Cursor::new(reordered.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        reordered.bytes.len() as u64,
        request(&reordered, 512),
    )
    .expect_err("page order mutation");
    assert!(
        !matches!(
            denial,
            BoundedCheckpointBackupDenial::InternalDigestMismatch
        ),
        "the rehashed page-order mutation must reach semantic validation: {denial:?}"
    );
}

#[test]
fn rehashed_second_page_frontier_mutations_are_denied() {
    let original = raw_fixture_with_pages(1, &[10, 10]);
    let mutations = [
        (78 + 32 + 8, 3, "second page identity"),
        (78 + 32 + 16, 2, "second page generation"),
        (78 + 32 + 24, 11, "second page LSN"),
    ];
    for (offset, value, field) in mutations {
        let fixture = rehashed_u64_fixture(&original, offset, value);
        let mut reader = Cursor::new(fixture.bytes.as_slice());
        let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
            &mut reader,
            fixture.bytes.len() as u64,
            request(&fixture, 512),
        )
        .expect_err(field);
        assert!(
            matches!(denial, BoundedCheckpointBackupDenial::BindingMismatch),
            "{field} mutation must not redefine the owner frontier: {denial:?}"
        );
    }
}

#[test]
fn exact_buffer_floor_denies_before_identity_allocation() {
    let fixture = raw_fixture(1, 10);
    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, fixture.peak_buffer_bytes() as usize - 1),
    )
    .expect_err("header plus footer is not enough for identity bytes");

    assert!(matches!(
        denial,
        BoundedCheckpointBackupDenial::BufferTooSmall
    ));
}

#[test]
fn multi_page_retention_is_charged_by_the_buffer_bound() {
    let fixture = raw_fixture_with_pages(1, &[10, 10]);
    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let denial = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, fixture.peak_buffer_bytes() as usize - 1),
    )
    .expect_err("retained page frontier must consume the buffer budget");
    assert!(matches!(
        denial,
        BoundedCheckpointBackupDenial::BufferTooSmall
    ));

    let mut reader = Cursor::new(fixture.bytes.as_slice());
    let observation = verify_bounded_checkpoint_backup_artifact_from_reader(
        &mut reader,
        fixture.bytes.len() as u64,
        request(&fixture, fixture.peak_buffer_bytes() as usize),
    )
    .expect("exact multi-page buffer bound");
    assert_eq!(
        observation.decoder_allocation_bytes(),
        fixture.decoder_allocation_bytes()
    );
}
