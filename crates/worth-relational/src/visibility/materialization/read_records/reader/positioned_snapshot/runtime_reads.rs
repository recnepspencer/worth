//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl VisibilityReadContext<'_> {
    /// Admit the selected visibility-basis copies and canonical route lookup
    /// before asking the unchanged positioned-snapshot owner to resolve them.
    /// The registry, rather than caller-supplied branch text, supplies the
    /// exact owned branch width. Snapshot IDs cannot be reused after release.
    /// Metadata read: one selected root and route position, with no row iteration; no execution request is needed.
    pub fn positioned_snapshot_admitted<Stop>(
        &self,
        handle: &SnapshotHandle,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<PositionedRelationalSnapshot, RelationalSnapshotPositionAdmissionStop<Stop>> {
        use RelationalSnapshotPositionAdmissionStop as Denial;
        // One runtime identity visit and up to two active/published fixed-key
        // registry lookups precede the selected-width read. This follows the
        // Native admitted-projection hash-lookup operation convention; neither
        // probe materializes a binding.
        admit(3, 0).map_err(Denial::Admission)?;
        if handle.runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(Denial::Position(
                SnapshotPositionDenial::SnapshotUnavailable,
            ));
        }
        let branch_bytes = self
            .runtime
            .visibility
            .selected_snapshot_branch_bytes(handle.snapshot_id())
            .ok_or(Denial::Position(
                SnapshotPositionDenial::SnapshotUnavailable,
            ))?;
        // Active resolution clones the binding's basis key and branch metadata,
        // clones its basis again after releasing the handle registry lock, and
        // copies that key into the positioned result. Published resolution
        // performs fewer copies. Seven further owner operation units cover up
        // to two active/published resolution lookups, one canonical-route
        // lookup, and four basis/Arc/result metadata visits.
        let bytes = branch_bytes
            .checked_mul(4)
            .ok_or(Denial::AccountingOverflow)?;
        let work = bytes.checked_add(7).ok_or(Denial::AccountingOverflow)?;
        admit(
            u64::try_from(work).map_err(|_| Denial::AccountingOverflow)?,
            u64::try_from(bytes).map_err(|_| Denial::AccountingOverflow)?,
        )
        .map_err(Denial::Admission)?;
        self.positioned_snapshot(handle).map_err(Denial::Position)
    }

    /// Resolve the selected snapshot root first, then its canonical stream position.
    /// This never admits a write basis or substitutes the current branch head.
    /// Metadata read: one selected root and route position, with no row iteration; no execution request is needed.
    pub fn positioned_snapshot(
        &self,
        handle: &SnapshotHandle,
    ) -> Result<PositionedRelationalSnapshot, SnapshotPositionDenial> {
        let basis = crate::visibility::snapshot_states::resolution::resolve_snapshot_basis(
            self.runtime,
            handle,
        )
        .ok_or(SnapshotPositionDenial::SnapshotUnavailable)?;
        let commit_id = basis.commit_id();
        let position = match commit_id {
            Some(commit_id) => Some(
                self.runtime
                    .history
                    .canonical_stream_position(commit_id)
                    .ok_or(SnapshotPositionDenial::SelectedCommitPositionUnavailable {
                        commit_id,
                    })?,
            ),
            None => None,
        };
        Ok(PositionedRelationalSnapshot {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            branch_id: basis.branch_id().clone(),
            root_id: basis.root_id(),
            version_id: basis.version_id(),
            commit_id,
            position,
        })
    }
}
