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
        let indexes = self.pin_indexes(&basis, context.schema_version, selected_indexes)?;
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

impl VisibilityReadContext<'_> {
    /// Pin an already issued exact projection without opening a tracked snapshot.
    pub(crate) fn pin_view_read_capability(
        &self,
        view: &crate::runtime::VisibilityProjectionView,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead<()>, RelationalPinnedReadAdmissionDenial> {
        use RelationalPinnedReadAdmissionDenial as Denial;
        request.check_live().map_err(Denial::Request)?;
        if !view.is_from_runtime(self.runtime) {
            return Err(Denial::SnapshotUnavailable);
        }
        let basis = view
            .retain_exact_basis()
            .ok_or(Denial::SnapshotUnavailable)?;
        let (schema_version, descriptor_semantics_version, evidence_basis) =
            super::super::snapshot_reads::query_schema_context_for_root(basis.root());
        let context = QueryPlanContextId {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            // Observations carry an exact root, but no operational snapshot identity.
            // This metadata cannot open a snapshot/plan door on a `()` pin.
            snapshot_id: crate::snapshots::data::SnapshotId(0),
            version_id: basis.version_id(),
            schema_version,
            descriptor_semantics_version,
            evidence_basis,
        };
        let indexes = self.pin_indexes(&basis, schema_version, selected_indexes)?;
        let (current_version, instrumentation, symbols) = view.retain_pinning_metadata();
        Ok(RelationalPinnedRead {
            current_version,
            instrumentation,
            snapshot: (),
            basis,
            context,
            symbols,
            indexes,
        })
    }

    fn pin_indexes(
        &self,
        basis: &VisibilitySnapshotBasis,
        schema: crate::schema::data::SchemaVersionId,
        selected_indexes: &[DerivedIndexId],
    ) -> Result<BTreeMap<DerivedIndexId, PinnedIndex>, RelationalPinnedReadAdmissionDenial> {
        use RelationalPinnedReadAdmissionDenial as Denial;
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
                schema,
            );
            indexes.insert(
                index,
                PinnedIndex {
                    definition,
                    generation,
                },
            );
        }
        Ok(indexes)
    }
}
