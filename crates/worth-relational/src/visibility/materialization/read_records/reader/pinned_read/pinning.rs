use super::*;
use crate::visibility::materialization::read_records::reader::VisibilityReadContext;
use crate::visibility::snapshot_states::resolve_snapshot_basis;

impl VisibilityReadContext<'_> {
    /// Capture native immutable custody while the installed owner is borrowed.
    /// Only selected index handles are retained. Index absence remains cold;
    /// no later read is permitted to consult a current generation registry.
    pub fn pin_read_capability(
        &self,
        handle: &SnapshotHandle,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead, RelationalPinnedReadAdmissionDenial> {
        use RelationalPinnedReadAdmissionDenial as Denial;
        request.check_live().map_err(Denial::Request)?;
        let snapshot = self
            .resolved_snapshot_handle(handle)
            .ok_or(Denial::SnapshotUnavailable)?;
        let context = self
            .query_plan_context(&snapshot)
            .ok_or(Denial::SnapshotUnavailable)?;
        let basis =
            resolve_snapshot_basis(self.runtime, &snapshot).ok_or(Denial::SnapshotUnavailable)?;
        let mut indexes = BTreeMap::new();
        for &index in selected_indexes {
            let definition = self
                .runtime
                .indexes
                .definition(index)
                .ok_or(Denial::IndexNotInstalled(index))?;
            let generation = self.runtime.indexes.exact_generation(
                index,
                definition.branch_scoped.then_some(basis.branch_id()),
                basis.version_id(),
                context.schema_version,
            );
            indexes.insert(
                index,
                PinnedIndex {
                    definition,
                    generation,
                },
            );
        }
        // StringInterner's im maps clone shared nodes, never their entries.
        let symbols = self.runtime.services.symbols.interner_snapshot();
        Ok(RelationalPinnedRead {
            current_version: self.runtime.current_version_id(),
            instrumentation: self.runtime.services.instrumentation.clone(),
            snapshot,
            basis,
            context,
            symbols,
            indexes,
        })
    }
}
