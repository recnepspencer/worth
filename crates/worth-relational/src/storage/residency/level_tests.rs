//! Compile-time separation of a Head lease from full historical substrate.
use super::super::{Full, Head, RelationalRegionHeadView, RelationalResidencyLease};
use crate::storage::overlay::PartitionState;

#[test]
fn head_cannot_read_or_infallibly_promote_to_historical_substrate() {
    // Type inference has exactly one choice only while the prohibited trait
    // implementation does not exist. Adding one makes this test fail to build.
    trait WithoutHistory<A> {
        fn witness() {}
    }
    impl<T: ?Sized> WithoutHistory<()> for T {}
    impl<T: ?Sized + std::ops::Deref<Target = PartitionState>> WithoutHistory<u8> for T {}
    let _ = <RelationalResidencyLease<Head> as WithoutHistory<_>>::witness;

    trait WithoutPromotion<A> {
        fn witness() {}
    }
    impl<T> WithoutPromotion<()> for T {}
    impl<T: Into<RelationalResidencyLease<Full>>> WithoutPromotion<u8> for T {}
    let _ = <RelationalResidencyLease<Head> as WithoutPromotion<_>>::witness;

    trait WithoutPartition<A> {
        fn witness() {}
    }
    impl<T> WithoutPartition<()> for T {}
    impl<T: AsRef<PartitionState>> WithoutPartition<u8> for T {}
    let _ = <RelationalRegionHeadView<'_> as WithoutPartition<_>>::witness;

    fn historical_substrate<T: std::ops::Deref<Target = PartitionState>>() {}
    historical_substrate::<RelationalResidencyLease<Full>>();
}
