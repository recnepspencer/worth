//! Bounded kind scans over the view's own branch root. A fork's view reads
//! the fork's writes and never its parent's later ones.

use crate::identity::data::KindId;
use crate::visibility::materialization::read_records::reader::{
    BoundedEntityKindTruthRead, BoundedRelationKindTruthRead, EntityKindTruthReadLimitExceeded,
    KindScanVisibility, RelationKindTruthReadDenial,
};
use crate::visibility::snapshot_states::SnapshotStateBasis;

use super::historical_basis_reads::HistoricalProjectionReader;
use super::VisibilityProjectionView;

impl VisibilityProjectionView {
    /// The entities of one kind in this view, refused before spending more
    /// than `maximum_work_units` on examined slots and materialized records.
    pub fn bounded_entities_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedEntityKindTruthRead, EntityKindTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.demand_all_full(execution)?;
        crate::visibility::materialization::read_records::read_work::scan_with_memory(
            execution,
            |context, memory| match &self.basis {
                SnapshotStateBasis::Exact(basis) => {
                    self.reader().bounded_entities_of_kind_in_state(
                        basis.root().as_ref(),
                        basis.root().schema_authority().registry(),
                        kind_id,
                        KindScanVisibility::Live,
                        maximum_work_units,
                        &mut |units| context.checkpoint(units),
                        memory,
                    )
                }
                SnapshotStateBasis::Historical(basis) => {
                    let historical = HistoricalProjectionReader::new(self, basis);
                    self.reader().bounded_entities_of_kind_in_state(
                        &historical.storage(),
                        historical.registry(),
                        kind_id,
                        KindScanVisibility::for_selected_version(
                            self.current_version,
                            self.version_id(),
                        ),
                        maximum_work_units,
                        &mut |units| context.checkpoint(units),
                        memory,
                    )
                }
            },
            |read| read.owned_bytes(),
        )
    }

    /// The relations of one kind in this view, refused before spending more
    /// than `maximum_work_units` on examined slots and materialized records.
    pub fn bounded_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedRelationKindTruthRead, RelationKindTruthReadDenial>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.demand_all_full(execution)?;
        crate::visibility::materialization::read_records::read_work::scan_with_memory(
            execution,
            |context, memory| match &self.basis {
                SnapshotStateBasis::Exact(basis) => {
                    self.reader().bounded_relations_of_kind_in_state(
                        basis.root().as_ref(),
                        basis.root().schema_authority().registry(),
                        kind_id,
                        KindScanVisibility::Live,
                        maximum_work_units,
                        &mut |units| context.checkpoint(units),
                        memory,
                    )
                }
                SnapshotStateBasis::Historical(basis) => {
                    let historical = HistoricalProjectionReader::new(self, basis);
                    self.reader().bounded_relations_of_kind_in_state(
                        &historical.storage(),
                        historical.registry(),
                        kind_id,
                        KindScanVisibility::for_selected_version(
                            self.current_version,
                            self.version_id(),
                        ),
                        maximum_work_units,
                        &mut |units| context.checkpoint(units),
                        memory,
                    )
                }
            },
            |read| read.owned_bytes(),
        )
    }
}
