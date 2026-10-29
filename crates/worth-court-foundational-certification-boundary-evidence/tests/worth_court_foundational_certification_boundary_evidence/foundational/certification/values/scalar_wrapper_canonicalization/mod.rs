use {worth_foundational::CanonicalBigInt, worth_foundational::CanonicalF32, worth_foundational::CanonicalF64, worth_foundational::CanonicalRational, worth_foundational::CanonicalTime};

#[test]
fn canonical_wrappers_reject_or_normalize_hostile_scalar_edges() {
    assert!(worth_foundational::CanonicalRational::new(worth_foundational::CanonicalBigInt::new("1"), worth_foundational::CanonicalBigInt::new("0")).is_none());
    assert!(
        worth_foundational::CanonicalRational::new(worth_foundational::CanonicalBigInt::new("1"), worth_foundational::CanonicalBigInt::new("00")).is_none()
    );
    assert!(
        worth_foundational::CanonicalRational::new(worth_foundational::CanonicalBigInt::new("1"), worth_foundational::CanonicalBigInt::new("+0")).is_none()
    );
    assert!(
        worth_foundational::CanonicalRational::new(worth_foundational::CanonicalBigInt::new("1"), worth_foundational::CanonicalBigInt::new("-0")).is_none()
    );
    assert!(worth_foundational::CanonicalTime::new(worth_foundational::CanonicalTime::NANOS_PER_DAY).is_none());

    let nan_a = worth_foundational::CanonicalF32::from_bits(0x7fc0_0001);
    let nan_b = worth_foundational::CanonicalF32::from_bits(0x7fc0_ffff);
    let nan_c = worth_foundational::CanonicalF64::from_bits(0x7ff8_0000_0000_0001);
    let nan_d = worth_foundational::CanonicalF64::from_bits(0x7ff8_ffff_ffff_ffff);

    assert_eq!(nan_a, nan_b);
    assert_eq!(nan_c, nan_d);
}
