use std::mem::size_of;

use super::{
    budget_fields, write_budget_field, WorthQueryGraphReadBudget, WorthQueryGraphReadBudgetDigest,
};
use crate::admission_digest::{hash_parts_with_digests_admitted, AdmittedHashStop};
use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;
use crate::graph_read_access::digest_text::admitted_digest_text;

impl WorthQueryGraphReadBudget {
    /// Prepare the existing budget identity under the caller's carried meter.
    /// Resource refusal returns before the next rendering or hashing phase.
    pub fn bounded_admitted<Stop>(
        max_inline_index_bytes: usize,
        max_inline_result_bytes: usize,
        max_inline_intermediate_set_size: usize,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryCanonicalIdentityStop<Stop>> {
        use WorthQueryCanonicalIdentityStop::{AccountingOverflow, Admission};
        let slot_work = u64::try_from(3 * size_of::<String>()).map_err(|_| AccountingOverflow)?;
        admit(slot_work + 3, 0).map_err(Admission)?;
        let fields = budget_fields(
            max_inline_index_bytes,
            max_inline_result_bytes,
            max_inline_intermediate_set_size,
        );
        let render = |(label, value): (&str, usize), admit: &mut _| {
            // Decimal digits are bounded by the integer's bit width. This
            // funds the count pass before discovering its exact output size.
            let work = u64::try_from(label.len())
                .map_err(|_| AccountingOverflow)?
                .checked_add(u64::from(usize::BITS) + 2)
                .ok_or(AccountingOverflow)?;
            admitted_digest_text(
                work,
                |output| write_budget_field(output, label, value),
                admit,
            )
            .map_err(WorthQueryCanonicalIdentityStop::from)
        };
        let parts = [
            render(fields[0], admit)?,
            render(fields[1], admit)?,
            render(fields[2], admit)?,
        ];
        let digest =
            hash_parts_with_digests_admitted(&parts, &[], admit).map_err(|stop| match stop {
                AdmittedHashStop::Admission(stop) => Admission(stop),
                AdmittedHashStop::AccountingOverflow => AccountingOverflow,
            })?;
        Ok(Self {
            digest: WorthQueryGraphReadBudgetDigest(digest),
            max_inline_index_bytes,
            max_inline_result_bytes,
            max_inline_intermediate_set_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_budget_keeps_identity_and_stops_at_the_last_hash_claim() {
        for limits in [(5120, 2048, 512), (0, 0, 0), (usize::MAX, 1, usize::MAX)] {
            let ordinary = WorthQueryGraphReadBudget::bounded(limits.0, limits.1, limits.2);
            let mut claims = Vec::new();
            let admitted = WorthQueryGraphReadBudget::bounded_admitted(
                limits.0,
                limits.1,
                limits.2,
                &mut |work, bytes| {
                    claims.push((work, bytes));
                    Ok::<_, usize>(())
                },
            )
            .unwrap();
            assert_eq!(ordinary, admitted);
            let last = claims.len() - 1;
            let mut calls = 0;
            let stopped = WorthQueryGraphReadBudget::bounded_admitted(
                limits.0,
                limits.1,
                limits.2,
                &mut |_, _| {
                    let claim = calls;
                    calls += 1;
                    if claim == last {
                        Err(claim)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(
                matches!(stopped, Err(WorthQueryCanonicalIdentityStop::Admission(claim)) if claim == last)
            );
            assert_eq!(calls, claims.len());
        }
    }
}
