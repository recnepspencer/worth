//! Sparse shell and selected adoption paths, without scanning retained payload.
use super::{RecordArena, RecordKind};
use crate::storage::substrate::SharedColumn;
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn retention_inspection_work_bound(&self) -> Option<u64> {
        // Retired header, three policy pin lookups, and the old lifecycle.
        let mut work = (self.slots.navigation_work_bound() as u64).checked_mul(5)?;
        work = work.checked_add(self.retired_at.lookup_work_bound())?;
        work = work.checked_add(self.lifecycle.lookup_work_bound())?;
        for pins in [&self.branch_pins, &self.replay_pins, &self.snapshot_pins] {
            work = work.checked_add(pins.lookup_work_bound())?;
        }
        Some(work)
    }

    pub(crate) fn retention_write_work_bound(&self, reset: bool) -> Option<u64> {
        let mut work = self.lifecycle.overwrite_work_bound().checked_add(
            // One quote lookup, lifecycle write, and (for reset) arena reset.
            (self.slots.navigation_work_bound() as u64).checked_mul(if reset { 3 } else { 2 })?,
        )?;
        if reset {
            macro_rules! columns {
                ($($column:ident),+ $(,)?) => { $(
                    work = work.checked_add(self.$column.overwrite_work_bound())?;
                )+ };
            }
            columns!(
                kind_ids,
                extra,
                aspect_versions,
                field_revisions,
                diagnostics_enrichment,
                retired_at
            );
            for pins in [&self.branch_pins, &self.replay_pins, &self.snapshot_pins] {
                work = work.checked_add(pins.reset_work_bound()?)?;
            }
        }
        Some(work)
    }
    pub(crate) fn retention_refresh_bound(&self, slot: usize) -> Option<u64> {
        self.lifecycle
            .overwrite_allocation_bound(self.physical_index(slot)?)
    }

    /// Reset writes new scalar/empty values, never clones the displaced payload.
    pub(crate) fn retention_reset_bound(&self, slot: usize) -> Option<u64> {
        let physical = self.physical_index(slot)?;
        let mut bytes = 0u64;
        macro_rules! columns {
            ($($column:ident),+ $(,)?) => { $(
                bytes = bytes.checked_add(self.$column.overwrite_allocation_bound(physical)?)?;
            )+ };
        }
        columns!(
            lifecycle,
            kind_ids,
            extra,
            aspect_versions,
            field_revisions,
            diagnostics_enrichment,
            retired_at
        );
        for pins in [&self.branch_pins, &self.replay_pins, &self.snapshot_pins] {
            bytes = bytes.checked_add(pins.reset_allocation_bound(physical)?)?;
        }
        Some(bytes)
    }

    pub(crate) fn sparse_shell_allocation_bound(&self) -> Option<u64> {
        [
            SharedColumn::<SharedColumn<K::Meta>>::sparse_default_allocation_bytes(),
            SharedColumn::<K::Extra>::sparse_default_allocation_bytes(),
            SharedColumn::<BTreeMap<Symbol, u64>>::sparse_default_allocation_bytes(),
            SharedColumn::<
                Option<BTreeMap<(Symbol, Symbol), crate::storage::data::RelationalFieldRevision>>,
            >::sparse_default_allocation_bytes(),
            SharedColumn::<BTreeMap<Symbol, String>>::sparse_default_allocation_bytes(),
        ]
        .into_iter()
        .try_fold(0u64, u64::checked_add)
    }
    pub(crate) fn sparse_slot_adoption_bound(&self, slot: usize) -> Option<u64> {
        let physical = self.physical_index(slot)?;
        // Temporary selected shell and destination coexist. Only shared value
        // handles move: arbitrary aspect/history payload is never cloned here.
        let mut bytes = 0u64;
        macro_rules! columns {
            ($($column:ident),+ $(,)?) => { $(
                bytes = bytes.checked_add(self.$column.adoption_allocation_bound(physical)?)?;
            )+ };
        }
        columns!(
            partition_ids,
            generations,
            lifecycle,
            kind_ids,
            metadata_history,
            created_at,
            retired_at,
            extra,
            aspect_versions,
            field_revisions,
            diagnostics_enrichment
        );
        // The destination already has this exact protected generation and
        // header. Its bitsets/pins stay unchanged; only value-owner paths copy.
        Some(bytes)
    }
}
