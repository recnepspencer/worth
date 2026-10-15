use serde::{Deserialize, Serialize};

use super::{StoredSymbolText, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SymbolTableSnapshot {
    pub entries: Vec<(Symbol, StoredSymbolText)>,
}

impl SymbolTableSnapshot {
    /// Import a current native symbol configuration under the original request.
    /// In-memory Clone already retains actual custody; context-free Deserialize
    /// deliberately refuses managed text instead of staging an unpaid String.
    pub fn readmit_current(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, crate::durability::data::RelationalRecoveryStop> {
        execution.check_live().map_err(|stop| {
            crate::durability::data::RelationalRecoveryStop::Request(stop.into())
        })?;
        let wire: super::SymbolSnapshotWire<'_> =
            rmp_serde::from_slice(bytes).map_err(|error| {
                crate::durability::data::DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    format!("native symbol configuration: {error}"),
                )
            })?;
        wire.readmit(execution)
            .map_err(super::field_text_recovery_stop)
    }

    pub fn merge_new_entries(&mut self, mut new_entries: Vec<(Symbol, StoredSymbolText)>) {
        if new_entries.is_empty() {
            return;
        }

        new_entries.sort_by(|left, right| left.1.cmp(&right.1));
        let existing = std::mem::take(&mut self.entries);
        let mut existing = existing.into_iter().peekable();
        let mut incoming = new_entries.into_iter().peekable();
        let mut merged = Vec::new();

        while let (Some(current), Some(next)) = (existing.peek(), incoming.peek()) {
            if current.1 <= next.1 {
                merged.push(existing.next().expect("peeked existing entry"));
            } else {
                merged.push(incoming.next().expect("peeked incoming entry"));
            }
        }

        merged.extend(existing);
        merged.extend(incoming);
        self.entries = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTableSnapshot;
    use crate::symbols::data::Symbol;

    #[test]
    fn snapshot_merge_new_entries_preserves_order_without_recloning_existing_entries() {
        let mut snapshot = SymbolTableSnapshot {
            entries: vec![(Symbol(2), "beta".into()), (Symbol(4), "delta".into())],
        };

        snapshot.merge_new_entries(vec![
            (Symbol(1), "alpha".into()),
            (Symbol(3), "charlie".into()),
        ]);

        assert_eq!(
            snapshot
                .entries
                .iter()
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta", "charlie", "delta"]
        );
    }
}
