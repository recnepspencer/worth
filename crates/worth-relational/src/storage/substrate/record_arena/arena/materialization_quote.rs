//! Sparse shell and selected adoption paths, without scanning retained payload.
use super::{RecordArena, RecordKind};
use crate::storage::substrate::SharedColumn;
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;

impl<K: RecordKind> RecordArena<K> {
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
