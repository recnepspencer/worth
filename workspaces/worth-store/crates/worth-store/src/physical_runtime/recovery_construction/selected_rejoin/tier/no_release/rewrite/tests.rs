use super::*;
use worth_store_wal::LogSequenceNumber;

const OPERATION: [u8; 32] = [3; 32];
const FINGERPRINT: [u8; 32] = [4; 32];
const GROUP: [u8; 32] = [5; 32];

fn range(start: u64, end: u64) -> WalLsnRange {
    WalLsnRange::new(LogSequenceNumber::new(start), LogSequenceNumber::new(end)).unwrap()
}

fn rewrite(
    fingerprint: [u8; 32],
    group: [u8; 32],
    source_root: u64,
    source_generation: u64,
    destination_generation: u64,
    page_lsn: u64,
    result_root: u64,
) -> Vec<u8> {
    PhysicalRewriteRedo::new(
        fingerprint,
        group,
        source_root,
        source_generation,
        64,
        32,
        [9; 32],
        destination_generation,
        128,
        page_lsn,
        [6; 32],
        11,
        12,
        result_root,
    )
    .unwrap()
    .encode()
}

fn admit(
    bytes: &[u8],
    member_group: [u8; 32],
    lsn: WalLsnRange,
    operations: &[([u8; 32], [u8; 32])],
    selected_root: u64,
) -> Result<bool, Denial> {
    admit_selected_rewrite(
        bytes,
        OPERATION,
        member_group,
        lsn,
        operations.iter().copied(),
        selected_root,
    )
}

fn denied(result: Result<bool, Denial>) {
    assert!(matches!(&result, Err(Denial::WalFate)), "{result:?}");
}

#[test]
fn selected_rewrite_requires_exact_sampled_operation_and_group() {
    let bytes = rewrite(FINGERPRINT, GROUP, 4, 7, 8, 11, 5);
    let lsn = range(11, 12);
    assert!(admit(&bytes, GROUP, lsn, &[(OPERATION, FINGERPRINT)], 9).unwrap());
    denied(admit(&bytes, GROUP, lsn, &[], 9));
    denied(admit(
        &bytes,
        GROUP,
        lsn,
        &[(OPERATION, FINGERPRINT), (OPERATION, FINGERPRINT)],
        9,
    ));
    denied(admit(&bytes, GROUP, lsn, &[(OPERATION, [0; 32])], 9));
    denied(admit(&bytes, [0; 32], lsn, &[(OPERATION, FINGERPRINT)], 9));
}

#[test]
fn selected_rewrite_requires_one_lsn_and_bounded_successor_lineage() {
    let operations = [(OPERATION, FINGERPRINT)];
    let bytes = rewrite(FINGERPRINT, GROUP, 4, 7, 8, 11, 5);
    denied(admit(&bytes, GROUP, range(11, 13), &operations, 9));
    denied(admit(
        &rewrite(FINGERPRINT, GROUP, 4, 7, 8, 12, 5),
        GROUP,
        range(11, 12),
        &operations,
        9,
    ));
    for (source_root, source_generation, destination_generation, result_root, selected) in [
        (0, 7, 8, 1, 9),
        (4, 0, 1, 5, 9),
        (4, 7, 9, 5, 9),
        (4, 7, 8, 6, 9),
        (4, 7, 8, 5, 4),
    ] {
        denied(admit(
            &rewrite(
                FINGERPRINT,
                GROUP,
                source_root,
                source_generation,
                destination_generation,
                11,
                result_root,
            ),
            GROUP,
            range(11, 12),
            &operations,
            selected,
        ));
    }
}

#[test]
fn only_a_distinct_domain_falls_through_to_strict_canonical_admission() {
    let operations = [(OPERATION, FINGERPRINT)];
    let bytes = rewrite(FINGERPRINT, GROUP, 4, 7, 8, 11, 5);
    denied(admit(
        &bytes[..bytes.len() - 1],
        GROUP,
        range(11, 12),
        &operations,
        9,
    ));
    let mut other_domain = bytes;
    other_domain[8] ^= 1;
    assert!(!admit(&other_domain, GROUP, range(11, 12), &operations, 9).unwrap());
}
