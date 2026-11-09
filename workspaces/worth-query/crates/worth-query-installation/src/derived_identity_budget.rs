//! Internal identity allowance carried from admitted declaration work.
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use worth_foundational::facade::CanonicalDigestWorkBudget;

/// One derived identity may expand each admitted entry/byte at most sixteen
/// times. See the installation README for the component-wise injection proof.
/// The shared 64 MiB declaration ceiling makes both multiplications fit even
/// on 32-bit targets; callers cannot supply work or widen this allowance.
pub(crate) fn derived_identity_budget(
    admitted: WorthQueryCanonicalWorkEvidence,
) -> CanonicalDigestWorkBudget {
    CanonicalDigestWorkBudget::new(
        admitted
            .canonical_entries()
            .checked_mul(16)
            .expect("admitted declaration entries fit the derived allowance"),
        admitted
            .canonical_encoded_bytes()
            .checked_mul(16)
            .expect("admitted declaration bytes fit the derived allowance"),
    )
    .expect("admitted declarations have nonempty canonical meaning")
}
