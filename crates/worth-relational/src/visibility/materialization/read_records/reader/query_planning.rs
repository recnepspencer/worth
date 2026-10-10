use super::*;

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Metadata read: planning examines descriptors and supplied targets, never stored rows; no execution request is needed.
    pub fn plan_query_packet(
        &self,
        handle: &SnapshotHandle,
        packet: PlannedQueryPacket,
    ) -> Option<SnapshotPinnedQueryPlan> {
        let snapshot = self.resolved_snapshot_handle(handle)?;
        if packet.context_id != self.query_plan_context(&snapshot)? {
            return None;
        }
        let legality = if packet.requires_serial_reduction() {
            QueryParallelLegality::RequiresSerialReduction
        } else {
            QueryParallelLegality::LegalReadOnlySnapshot
        };
        let profitability = self.query_profitability(&snapshot, &packet);
        Some(SnapshotPinnedQueryPlan {
            packet,
            snapshot,
            legality,
            profitability,
        })
    }

    pub(super) fn query_profitability(
        &self,
        snapshot: &SnapshotHandle,
        packet: &PlannedQueryPacket,
    ) -> QueryParallelProfitability {
        if packet.target_count_hint <= 1 {
            return QueryParallelProfitability::SerialPreferred {
                reason: QuerySerialReason::TinyPacket,
            };
        }

        if let QueryScope::ExplicitTargets { targets } = &packet.scope {
            let touched_partitions = targets
                .iter()
                .map(|target| match target {
                    crate::transactions::data::RecordRef::Entity(entity_id) => {
                        entity_id.partition_id
                    }
                    crate::transactions::data::RecordRef::Relation(relation_id) => {
                        relation_id.partition_id
                    }
                })
                .collect::<std::collections::BTreeSet<_>>();
            if touched_partitions.len() > 1 {
                return QueryParallelProfitability::Profitable;
            }

            let read_packet = PlannedQueryPacket::explicit_targets(
                packet.label.clone(),
                packet.context_id.clone(),
                targets.to_vec(),
            );
            if let Some(read_plan) = self
                .runtime
                .storage_access()
                .plan_read_explicit_query_packet(snapshot, &read_packet)
            {
                let touched_chunk_count =
                    read_plan.entity_chunk_indexes.len() + read_plan.relation_chunk_indexes.len();
                if touched_chunk_count <= 1 {
                    return QueryParallelProfitability::SerialPreferred {
                        reason: QuerySerialReason::SingleChunkSurface,
                    };
                }
            }
        }

        if matches!(
            packet.locality,
            crate::query::data::QueryLocalityClass::CrossPartitionTraversal
        ) && packet.target_count_hint > 0
            && packet.target_count_hint <= 4
        {
            return QueryParallelProfitability::SerialPreferred {
                reason: QuerySerialReason::BroadCrossPartitionCoordination,
            };
        }

        QueryParallelProfitability::Profitable
    }
}
