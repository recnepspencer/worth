use super::admit_scratch_limit;

#[test]
fn decoder_headroom_denies_a_tight_grant_and_admits_its_larger_twin() {
    // Exercise the exact production admission arithmetic, including live
    // routing/free-space decoders and readers, not only collection entries.
    let admitted = |bytes| admit_scratch_limit(bytes, 8, 2, 2, 4096, 64).is_ok();
    let mut denied = 0_u64;
    let mut healthy = 1 << 20;
    assert!(admitted(healthy));
    while healthy - denied > 1 {
        let probe = denied + (healthy - denied) / 2;
        if admitted(probe) {
            healthy = probe;
        } else {
            denied = probe;
        }
    }
    assert!(!admitted(denied), "one byte below the full peak must deny");
    assert!(
        admitted(healthy),
        "the same work fits a one-byte larger grant"
    );
    assert!(
        healthy > 4 * 4096,
        "decoded nodes/readers consume real headroom"
    );
}
