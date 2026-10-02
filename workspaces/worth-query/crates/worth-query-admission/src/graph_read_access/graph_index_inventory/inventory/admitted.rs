use std::mem::size_of;

use crate::admission_digest::hash_parts_with_digests_admitted;
use crate::graph_read_access::{
    digest_order::sort_digest_keys_admitted, digest_text::admitted_digest_text,
};

use super::{WorthQueryGraphIndexInventory, WorthQueryGraphIndexSupportRow};
use crate::graph_read_access::graph_index_inventory::WorthQueryGraphIndexInventoryAdmissionStop;

impl WorthQueryGraphIndexInventory {
    pub fn from_rows_admitted<Stop>(
        rows: Vec<WorthQueryGraphIndexSupportRow>,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryGraphIndexInventoryAdmissionStop<Stop>> {
        use WorthQueryGraphIndexInventoryAdmissionStop as Refusal;
        admit(1, 0).map_err(Refusal::Admission)?;
        let count = rows.len();
        let pair_bytes = count
            .checked_mul(size_of::<(String, WorthQueryGraphIndexSupportRow)>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Refusal::AccountingOverflow)?;
        admit(pair_bytes, pair_bytes).map_err(Refusal::Admission)?;
        let mut keyed = Vec::new();
        keyed
            .try_reserve_exact(count)
            .map_err(|_| Refusal::AllocationUnavailable)?;
        for row in rows {
            admit(2, 0).map_err(Refusal::Admission)?;
            let kind = row.requirement_kind().as_str();
            let digest = row.digest();
            // NUL is outside both the fixed kind vocabulary and hex digest.
            // This is the same ordering as the ordinary (kind, digest) tuple.
            let count_work = kind
                .len()
                .checked_add(digest.len())
                .and_then(|n| n.checked_add(1))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Refusal::AccountingOverflow)?;
            let key = admitted_digest_text(
                count_work,
                |output| write!(output, "{kind}\0{digest}"),
                &mut *admit,
            )?;
            keyed.push((key, row));
        }
        let mut keyed = sort_digest_keys_admitted(keyed, admit).map_err(Refusal::from)?;
        admit(1, 0).map_err(Refusal::Admission)?;
        let count = keyed.len();
        let width_visits = u64::try_from(count).map_err(|_| Refusal::AccountingOverflow)?;
        admit(width_visits, 0).map_err(Refusal::Admission)?;
        let maximum_digest = keyed
            .iter()
            .map(|(_, row)| row.digest().len())
            .max()
            .unwrap_or(0);
        let comparison_work = count
            .checked_mul(
                maximum_digest
                    .checked_mul(2)
                    .ok_or(Refusal::AccountingOverflow)?,
            )
            .ok_or(Refusal::AccountingOverflow)?;
        let relocation_work = count
            .checked_mul(size_of::<(String, WorthQueryGraphIndexSupportRow)>())
            .and_then(|bytes| bytes.checked_mul(3))
            .ok_or(Refusal::AccountingOverflow)?;
        let dedup_work = comparison_work
            .checked_add(relocation_work)
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(Refusal::AccountingOverflow)?;
        admit(dedup_work, 0).map_err(Refusal::Admission)?;
        keyed.dedup_by(|left, right| left.1.digest() == right.1.digest());
        let row_bytes = keyed
            .len()
            .checked_mul(size_of::<WorthQueryGraphIndexSupportRow>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Refusal::AccountingOverflow)?;
        admit(row_bytes, row_bytes).map_err(Refusal::Admission)?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(keyed.len())
            .map_err(|_| Refusal::AllocationUnavailable)?;
        for (_, row) in keyed {
            rows.push(row);
        }

        let part_count = rows
            .len()
            .checked_add(1)
            .ok_or(Refusal::AccountingOverflow)?;
        let part_bytes = part_count
            .checked_mul(size_of::<String>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Refusal::AccountingOverflow)?;
        admit(part_bytes, part_bytes).map_err(Refusal::Admission)?;
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(part_count)
            .map_err(|_| Refusal::AllocationUnavailable)?;
        parts.push(admitted_digest_text(
            "worth_query_graph_index_inventory_v1".len() as u64,
            |output| output.write_str("worth_query_graph_index_inventory_v1"),
            &mut *admit,
        )?);
        for row in &rows {
            admit(1, 0).map_err(Refusal::Admission)?;
            let count_work = row
                .digest()
                .len()
                .checked_add(4)
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Refusal::AccountingOverflow)?;
            parts.push(admitted_digest_text(
                count_work,
                |output| row.write_digest_part(output),
                &mut *admit,
            )?);
        }
        let digest = hash_parts_with_digests_admitted(&parts, &[], &mut *admit)?;
        Ok(Self { digest, rows })
    }
}
