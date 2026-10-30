use crate::physical_blob_journeys::blob_reclaim_batch_boundary::FULL_INSPECTION_ALLOWANCE;

#[test]
fn reclaim_drop_set_limit_rejects_1025_ids() {
    assert_eq!(
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(4096).unwrap(),
            std::num::NonZeroU64::new(FULL_INSPECTION_ALLOWANCE).unwrap(),
            std::num::NonZeroU16::new(1025).unwrap(),
        ),
        Err(worth_store::physical_runtime::BlobReclaimLimitDenial::DropSetTooLarge)
    );
}
