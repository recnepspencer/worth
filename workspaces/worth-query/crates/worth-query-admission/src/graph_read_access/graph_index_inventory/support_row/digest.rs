use std::fmt::{self, Write};
use std::mem::size_of;

use crate::admission_digest::{hash_parts, hash_parts_with_digests_admitted};

use super::WorthQueryGraphIndexSupportRow;
use crate::graph_read_access::graph_index_inventory::WorthQueryGraphIndexInventoryAdmissionStop;

const PART_COUNT: usize = 14;

fn render_part(
    row: &WorthQueryGraphIndexSupportRow,
    part: usize,
    out: &mut dyn Write,
) -> fmt::Result {
    match part {
        0 => out.write_str("worth_query_graph_index_support_row_v1"),
        1 => write!(out, "requirement:{}", row.requirement_kind.as_str()),
        2 => write!(
            out,
            "direction:{}",
            row.supported_relation_direction
                .as_ref()
                .map_or("none", |value| value.as_str())
        ),
        3 => write!(
            out,
            "predicate:{}",
            row.supported_predicate_family
                .as_ref()
                .map_or("none", |value| value.as_str())
        ),
        4 => write!(
            out,
            "ordering:{}",
            row.supported_ordering_posture
                .as_ref()
                .map_or("none", |value| value.as_str())
        ),
        5 => write!(
            out,
            "requirement_lifecycle:{}",
            row.supported_requirement_lifecycle
                .as_ref()
                .map_or("none", |value| value.as_str())
        ),
        6 => write!(out, "owner:{}", row.lifecycle_owner.as_str()),
        7 => write!(out, "lifecycle:{}", row.lifecycle_class.as_str()),
        8 => write!(out, "rebuild:{}", row.rebuild_basis.as_str()),
        9 => write!(out, "invalidation:{}", row.invalidation_basis.as_str()),
        10 => write!(out, "complexity:{}", row.complexity_contract.as_str()),
        11 => write!(out, "posture:{}", row.posture.as_str()),
        12 => write!(out, "support_state:{}", row.support_state.as_str()),
        13 => write!(
            out,
            "owning_milestone:{}",
            row.owning_milestone.as_deref().unwrap_or("none")
        ),
        _ => unreachable!("support row digest has fourteen parts"),
    }
}

pub(super) fn ordinary_digest(row: &WorthQueryGraphIndexSupportRow) -> String {
    let mut parts = Vec::with_capacity(PART_COUNT);
    for part in 0..PART_COUNT {
        let mut text = String::new();
        render_part(row, part, &mut text).expect("String formatting cannot fail");
        parts.push(text);
    }
    hash_parts(&parts)
}

struct AdmittedCounter<'a, Stop> {
    bytes: usize,
    stop: Option<WorthQueryGraphIndexInventoryAdmissionStop<Stop>>,
    admit: &'a mut dyn FnMut(u64, u64) -> Result<(), Stop>,
}

impl<Stop> Write for AdmittedCounter<'_, Stop> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let Some(bytes) = self.bytes.checked_add(value.len()) else {
            self.stop = Some(WorthQueryGraphIndexInventoryAdmissionStop::AccountingOverflow);
            return Err(fmt::Error);
        };
        let Ok(work) = u64::try_from(value.len()) else {
            self.stop = Some(WorthQueryGraphIndexInventoryAdmissionStop::AccountingOverflow);
            return Err(fmt::Error);
        };
        if let Err(stop) = (self.admit)(work, 0) {
            self.stop = Some(WorthQueryGraphIndexInventoryAdmissionStop::Admission(stop));
            return Err(fmt::Error);
        }
        self.bytes = bytes;
        Ok(())
    }
}

pub(super) fn admitted_digest<Stop>(
    row: &WorthQueryGraphIndexSupportRow,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<String, WorthQueryGraphIndexInventoryAdmissionStop<Stop>> {
    use WorthQueryGraphIndexInventoryAdmissionStop as Refusal;
    // The fourteen selected fields are visited before rendering any part.
    admit(PART_COUNT as u64, 0).map_err(Refusal::Admission)?;
    let slots = PART_COUNT
        .checked_mul(size_of::<String>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Refusal::AccountingOverflow)?;
    admit(slots, slots).map_err(Refusal::Admission)?;
    let mut parts = Vec::new();
    parts
        .try_reserve_exact(PART_COUNT)
        .map_err(|_| Refusal::AllocationUnavailable)?;
    for part in 0..PART_COUNT {
        let mut counter = AdmittedCounter {
            bytes: 0,
            stop: None,
            admit,
        };
        if render_part(row, part, &mut counter).is_err() {
            return Err(counter.stop.unwrap_or(Refusal::AccountingOverflow));
        }
        let count_bytes = counter.bytes;
        drop(counter);
        let bytes = u64::try_from(count_bytes).map_err(|_| Refusal::AccountingOverflow)?;
        // The second render reads and writes the same exact part bytes.
        let render_work = bytes.checked_mul(2).ok_or(Refusal::AccountingOverflow)?;
        admit(render_work, bytes).map_err(Refusal::Admission)?;
        let mut text = String::with_capacity(count_bytes);
        render_part(row, part, &mut text).expect("String formatting cannot fail");
        debug_assert_eq!(text.len(), count_bytes);
        parts.push(text);
    }
    hash_parts_with_digests_admitted(&parts, &[], &mut *admit).map_err(Into::into)
}
