use super::*;
use worth_store_physical_format::{
    DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange, PersistedExtentCopyRecipe,
    PersistedRecordIdentity, PhysicalExtentCopyIntent, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

fn copy_fixture() -> (
    PhysicalRecordFormatDeclaration,
    PersistedExtentCopyRecipe,
    WalLsnRange,
    Box<[u8]>,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source = DurableExtentRecordPlacement::legacy_unknown(
        PersistedRecordIdentity::new([7; 16], 9).unwrap(),
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap()),
        40_000,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 53_248, 53_248).unwrap(),
    )
    .unwrap();
    let intent = PhysicalExtentCopyIntent::new(
        format,
        [19; 32],
        12,
        source,
        ExtentArenaRange::new(ExtentArenaId::new(8).unwrap(), 0, 53_248).unwrap(),
        4096,
        [23; 32],
    )
    .unwrap();
    let bytes = PhysicalExtentCopyRecord::Intent(intent).encode();
    let digest = Sha256::digest(&bytes).into();
    let recipe = PersistedExtentCopyRecipe::new(intent, 41, digest).unwrap();
    let range = WalLsnRange::new(LogSequenceNumber::new(41), LogSequenceNumber::new(42)).unwrap();
    (format, recipe, range, bytes.into_boxed_slice())
}

#[test]
fn no_release_custody_does_not_yet_admit_a_terminal_head_retirement_member() {
    let retirement = crate::physical_runtime::terminal_head_retirement_fixture::projection();
    assert!(matches!(
        tail_drop(retirement.operation()),
        Err(Denial::WalFate)
    ));
    assert!(matches!(
        tail_drop(&PersistedPhysicalRecoveryOperation::None),
        Ok(None)
    ));
}

#[test]
fn sampled_copy_publication_requires_one_exact_earlier_durable_intent() {
    let (format, recipe, range, bytes) = copy_fixture();
    assert!(!matches_exact_copy_intent(
        std::iter::empty(),
        recipe,
        format
    ));
    assert!(matches_exact_copy_intent(
        [(range, bytes.as_ref())].into_iter(),
        recipe,
        format
    ));
    assert!(!matches_exact_copy_intent(
        [(range, bytes.as_ref()), (range, bytes.as_ref())].into_iter(),
        recipe,
        format
    ));

    let mut substituted = bytes.to_vec();
    *substituted.last_mut().unwrap() ^= 1;
    assert!(!matches_exact_copy_intent(
        [(range, substituted.as_slice())].into_iter(),
        recipe,
        format
    ));
    let late = WalLsnRange::new(LogSequenceNumber::new(41), LogSequenceNumber::new(43)).unwrap();
    assert!(!matches_exact_copy_intent(
        [(late, bytes.as_ref())].into_iter(),
        recipe,
        format
    ));
}
