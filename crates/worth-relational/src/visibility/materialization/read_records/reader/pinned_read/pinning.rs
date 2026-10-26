use super::*;
use crate::visibility::materialization::read_records::reader::VisibilityReadContext;
use crate::visibility::snapshot_states::resolve_snapshot_basis;

impl VisibilityReadContext<'_> {
    /// Capture native immutable custody while the installed owner is borrowed.
    /// Only selected index handles are retained. Genuine index absence remains absent;
    /// no later read is permitted to consult a current generation registry.
    pub fn pin_read_capability(
        &self,
        handle: &SnapshotHandle,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead, RelationalPinnedReadAdmissionDenial> {
        self.pin_read_capability_checked(
            handle,
            selected_indexes,
            request,
            |_| Ok(()),
            std::convert::identity,
        )
    }

    /// Validate each semantic definition before touching its generation payload.
    pub(crate) fn pin_read_capability_checked<E>(
        &self,
        handle: &SnapshotHandle,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
        validate: impl Fn(&crate::indexes::data::DerivedIndexDefinition) -> Result<(), E>,
        denied: impl Fn(RelationalPinnedReadAdmissionDenial) -> E,
    ) -> Result<RelationalPinnedRead, E> {
        use RelationalPinnedReadAdmissionDenial as Denial;
        request
            .check_live()
            .map_err(|stop| denied(Denial::Request(stop.into())))?;
        let snapshot = self
            .resolved_snapshot_handle(handle)
            .ok_or_else(|| denied(Denial::SnapshotUnavailable))?;
        let basis = resolve_snapshot_basis(self.runtime, &snapshot)
            .ok_or_else(|| denied(Denial::SnapshotUnavailable))?;
        let (schema_version, _, _) =
            super::super::snapshot_reads::query_schema_context_for_root(basis.root());
        let indexes = self.pin_indexes(
            &basis,
            schema_version,
            selected_indexes,
            request,
            validate,
            &denied,
        )?;
        let context = self
            .query_plan_context(&snapshot)
            .ok_or_else(|| denied(Denial::SnapshotUnavailable))?;
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
    /// Pin an owner-issued exact projection without opening a tracked snapshot.
    ///
    /// The view must belong to this runtime and retain an exact native basis.
    /// Its immutable root, symbol dictionary and selected index handles stay
    /// shared with their original storage owners through the last pin clone.
    /// This capability has no operational snapshot identity and cannot open a
    /// snapshot query-plan door. The caller's request still checks liveness.
    pub fn pin_view_read_capability(
        &self,
        view: &crate::runtime::VisibilityProjectionView,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead<()>, RelationalPinnedReadAdmissionDenial> {
        self.pin_view_read_capability_checked(
            view,
            selected_indexes,
            request,
            |_| Ok(()),
            std::convert::identity,
        )
    }

    pub(crate) fn pin_view_read_capability_checked<E>(
        &self,
        view: &crate::runtime::VisibilityProjectionView,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
        validate: impl Fn(&crate::indexes::data::DerivedIndexDefinition) -> Result<(), E>,
        denied: impl Fn(RelationalPinnedReadAdmissionDenial) -> E,
    ) -> Result<RelationalPinnedRead<()>, E> {
        use RelationalPinnedReadAdmissionDenial as Denial;
        request
            .check_live()
            .map_err(|stop| denied(Denial::Request(stop.into())))?;
        if !view.is_from_runtime(self.runtime) {
            return Err(denied(Denial::SnapshotUnavailable));
        }
        let basis = view
            .retain_exact_basis()
            .ok_or_else(|| denied(Denial::SnapshotUnavailable))?;
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
        let indexes = self.pin_indexes(
            &basis,
            schema_version,
            selected_indexes,
            request,
            validate,
            denied,
        )?;
        let (current_version, instrumentation, _) = view.retain_pinning_metadata();
        let symbols = self.runtime.services.symbols.interner_snapshot();
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

    fn pin_indexes<E>(
        &self,
        basis: &VisibilitySnapshotBasis,
        schema: crate::schema::data::SchemaVersionId,
        selected_indexes: &[DerivedIndexId],
        request: &worth_execution::ExecutionRequest<'_, '_>,
        validate: impl Fn(&crate::indexes::data::DerivedIndexDefinition) -> Result<(), E>,
        denied: impl Fn(RelationalPinnedReadAdmissionDenial) -> E,
    ) -> Result<BTreeMap<DerivedIndexId, PinnedIndex>, E> {
        use RelationalPinnedReadAdmissionDenial as Denial;
        let mut definitions = BTreeMap::new();
        for &index in selected_indexes {
            let definition = self
                .runtime
                .indexes
                .definition(index)
                .ok_or_else(|| denied(Denial::IndexNotInstalled(index)))?;
            validate(&definition)?;
            definitions.insert(index, definition);
        }
        basis
            .root()
            .demand_all_full(request, &self.runtime.services.symbols)
            .map_err(|failure| denied(Denial::Request(failure.into())))?;
        if !definitions.is_empty() {
            self.runtime
                .indexes
                .definition_read_binding()
                .demand_checkpoint_image(request)
                .map_err(|failure| denied(Denial::Request(failure.into())))?;
        }
        let mut indexes = BTreeMap::new();
        for (index, definition) in definitions {
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
