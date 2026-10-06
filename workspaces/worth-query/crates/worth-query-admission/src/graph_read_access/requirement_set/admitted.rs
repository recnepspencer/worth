use super::canonical_basis::{self, RequirementDigestField};
use super::{
    WorthQueryGraphReadAccessRequirementCounters, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadAccessRequirementSetDigest,
};
use crate::canonical_identity_derivation::{
    WorthQueryCanonicalIdentityBasis, WorthQueryCanonicalIdentityStop,
};
use crate::graph_read_access::{
    digest_order::sort_digest_keys_admitted, digest_text::admitted_digest_text,
};
use std::mem::size_of;
use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

impl WorthQueryGraphReadAccessRequirementSet {
    pub(crate) fn new_admitted<Stop>(
        read_graph_digest: CanonicalDigestId,
        access_shape_digest: CanonicalDigestId,
        selectivity_shape_digest: CanonicalDigestId,
        rows: Vec<WorthQueryGraphReadAccessRequirementRow>,
        budget: CanonicalDigestWorkBudget,
        prior_work: WorthQueryCanonicalWorkEvidence,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryCanonicalIdentityStop<Stop>> {
        use WorthQueryCanonicalIdentityStop::{
            AccountingOverflow, Admission, AllocationUnavailable,
        };
        let count = u64::try_from(rows.len()).map_err(|_| AccountingOverflow)?;
        let pair_width = size_of::<(String, WorthQueryGraphReadAccessRequirementRow)>() as u64;
        admit(
            count.checked_mul(pair_width).ok_or(AccountingOverflow)?,
            count.checked_mul(pair_width).ok_or(AccountingOverflow)?,
        )
        .map_err(Admission)?;
        let mut keyed = Vec::new();
        keyed
            .try_reserve_exact(rows.len())
            .map_err(|_| AllocationUnavailable)?;
        for row in rows {
            admit(1, 0).map_err(Admission)?;
            let visits = row.digest_visit_work().ok_or(AccountingOverflow)?;
            let key =
                admitted_digest_text(visits, |output| row.write_digest_part(output), &mut *admit)?;
            keyed.push((key, row));
        }
        let keyed = sort_digest_keys_admitted(keyed, admit)?;
        let row_width = size_of::<WorthQueryGraphReadAccessRequirementRow>() as u64;
        let count = u64::try_from(keyed.len()).map_err(|_| AccountingOverflow)?;
        admit(
            count.checked_mul(row_width).ok_or(AccountingOverflow)?,
            count.checked_mul(row_width).ok_or(AccountingOverflow)?,
        )
        .map_err(Admission)?;
        let mut rows: Vec<WorthQueryGraphReadAccessRequirementRow> = Vec::new();
        rows.try_reserve_exact(keyed.len())
            .map_err(|_| AllocationUnavailable)?;
        let mut previous_key_width = 0_u64;
        let mut previous_inline_width = 0_u64;
        for (key, row) in keyed {
            // Row equality includes owned authority vectors. Their canonical
            // text covers payload bytes; fixed row storage covers scalars and
            // container headers. Compare the same Eq relation as ordinary dedup.
            admit(4, 0).map_err(Admission)?;
            let key_width = u64::try_from(key.len()).map_err(|_| AccountingOverflow)?;
            let inline_width = row.equality_inline_bytes().ok_or(AccountingOverflow)?;
            let equality = key_width
                .checked_add(previous_key_width)
                .and_then(|bytes| bytes.checked_add(inline_width))
                .and_then(|bytes| bytes.checked_add(previous_inline_width))
                .ok_or(AccountingOverflow)?;
            let visits = row.digest_visit_work().ok_or(AccountingOverflow)?;
            admit(equality.checked_add(visits).ok_or(AccountingOverflow)?, 0).map_err(Admission)?;
            if rows.last().is_none_or(|previous| previous != &row) {
                previous_key_width = key_width;
                previous_inline_width = inline_width;
                rows.push(row);
            }
        }
        // Counter construction performs the twelve existing kind scans.
        let visits = u64::try_from(rows.len())
            .map_err(|_| AccountingOverflow)?
            .checked_mul(12)
            .ok_or(AccountingOverflow)?;
        admit(visits, 0).map_err(Admission)?;
        let counters = WorthQueryGraphReadAccessRequirementCounters::from_rows(&rows);
        let (digest, work) = derive_digest_admitted(
            read_graph_digest,
            access_shape_digest,
            selectivity_shape_digest,
            &rows,
            budget,
            admit,
        )?;
        Ok(Self {
            digest: WorthQueryGraphReadAccessRequirementSetDigest(digest),
            read_graph_digest,
            access_shape_digest,
            selectivity_shape_digest,
            rows,
            counters,
            canonical_work: prior_work.combine(work),
        })
    }
}

fn derive_digest_admitted<Stop>(
    read_graph_digest: CanonicalDigestId,
    access_shape_digest: CanonicalDigestId,
    selectivity_shape_digest: CanonicalDigestId,
    rows: &[WorthQueryGraphReadAccessRequirementRow],
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<
    (CanonicalDigestId, WorthQueryCanonicalWorkEvidence),
    WorthQueryCanonicalIdentityStop<Stop>,
> {
    use WorthQueryCanonicalIdentityStop::{AccountingOverflow, Admission};
    let mut basis = WorthQueryCanonicalIdentityBasis::new_admitted(
        canonical_basis::DOMAIN,
        canonical_basis::VERSION,
        budget,
        admit,
    )?;
    admit(
        u64::try_from(rows.len())
            .map_err(|_| AccountingOverflow)?
            .checked_add(4)
            .ok_or(AccountingOverflow)?,
        0,
    )
    .map_err(Admission)?;
    canonical_basis::emit_fields(
        read_graph_digest,
        access_shape_digest,
        selectivity_shape_digest,
        rows,
        |field| match field {
            RequirementDigestField::Digest(locus, value) => {
                basis.digest_admitted(locus, value, admit)
            }
            RequirementDigestField::Unsigned(locus, value) => {
                basis.unsigned_admitted(locus, value, admit)
            }
            RequirementDigestField::Row(index, row) => {
                let locus = admitted_digest_text(
                    usize::BITS as u64 + 2,
                    |output| canonical_basis::write_row_locus(output, index),
                    &mut *admit,
                )?;
                let visits = row.digest_visit_work().ok_or(AccountingOverflow)?;
                let value = admitted_digest_text(
                    visits,
                    |output| row.write_digest_part(output),
                    &mut *admit,
                )?;
                basis.text_owned_admitted(locus, value, admit)
            }
        },
    )?;
    basis.derive_admitted(admit)
}
