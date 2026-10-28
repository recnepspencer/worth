use worth_foundational::{
    CanonicalBigInt, CanonicalF32, CanonicalF64, CanonicalRational, CanonicalTime,
};

#[test]
fn canonical_wrappers_reject_or_normalize_hostile_scalar_edges() {
    assert!(CanonicalRational::new(CanonicalBigInt::new("1"), CanonicalBigInt::new("0")).is_none());
    assert!(
        CanonicalRational::new(CanonicalBigInt::new("1"), CanonicalBigInt::new("00")).is_none()
    );
    assert!(
        CanonicalRational::new(CanonicalBigInt::new("1"), CanonicalBigInt::new("+0")).is_none()
    );
    assert!(
        CanonicalRational::new(CanonicalBigInt::new("1"), CanonicalBigInt::new("-0")).is_none()
    );
    assert!(CanonicalTime::new(CanonicalTime::NANOS_PER_DAY).is_none());

    let nan_a = CanonicalF32::from_bits(0x7fc0_0001);
    let nan_b = CanonicalF32::from_bits(0x7fc0_ffff);
    let nan_c = CanonicalF64::from_bits(0x7ff8_0000_0000_0001);
    let nan_d = CanonicalF64::from_bits(0x7ff8_ffff_ffff_ffff);

    assert_eq!(nan_a, nan_b);
    assert_eq!(nan_c, nan_d);
}
