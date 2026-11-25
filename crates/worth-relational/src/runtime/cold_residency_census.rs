//! Counted read-only observations of cold residency, pins, dictionaries and
//! counters at the Native owner.
//!
//! Every observation copies scalar counts, coordinates, status tags or weak
//! identities without triggering cold I/O, mutating runtime state, retaining
//! allocations, or granting any authority.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::data::BranchId;
use crate::identity::data::{EntityId, LineageId, PartitionId, VersionId};
use crate::performance::data::RuntimeComplexityCounters;
use crate::runtime::state::subsystems::history::history_canonical_routes::pending_canonical_publication_route_count;
use crate::runtime::RelationalRuntime;
use crate::storage::residency::{PartitionHeadProbe, PartitionProbe};
use crate::symbols::data::Symbol;

/// Read-only census of currently resident lineage nodes, events and frontiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalLineageResidencyObservation {
    pub resident_node_count: usize,
    pub resident_event_count: usize,
    pub node_identity_frontier: u64,
    pub event_identity_frontier: u64,
}
/// Read-only census of replay-pin retention state without triggering I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalReplayRetentionObservation {
    pub retained_versions: Vec<VersionId>,
    pub cached_historical_state_count: usize,
}
/// Residency status of one partition carrier without triggering cold I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRegionProbeStatus {
    Absent,
    Cold,
    Resident,
}
/// Head and Full residency status of one partition carrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalRegionResidencyObservation {
    pub head: RelationalRegionProbeStatus,
    pub full: RelationalRegionProbeStatus,
}
/// Read-only slot and generation census of one resident Full or Head partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalPartitionSlotCensus {
    pub entity_slot_count: usize,
    pub occupied_entity_slots: Vec<usize>,
    pub occupied_entity_generations: Vec<(usize, u32)>,
    pub relation_slot_count: usize,
    pub occupied_relation_slots: Vec<usize>,
}
/// Read-only copy of currently resident symbol interner entries (no cold I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalResidentSymbolDictionary {
    pub entries: Vec<(Symbol, String)>,
}
impl RelationalResidentSymbolDictionary {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn contains_text(&self, text: &str) -> bool {
        self.entries.iter().any(|(_, value)| value == text)
    }
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.entries
            .iter()
            .find(|(id, _)| *id == symbol)
            .map(|(_, value)| value.as_str())
    }
}
fn head_status(probe: PartitionHeadProbe<'_>) -> RelationalRegionProbeStatus {
    match probe {
        PartitionHeadProbe::Absent => RelationalRegionProbeStatus::Absent,
        PartitionHeadProbe::Cold => RelationalRegionProbeStatus::Cold,
        PartitionHeadProbe::Resident(_) => RelationalRegionProbeStatus::Resident,
    }
}
fn full_status(probe: PartitionProbe<'_>) -> RelationalRegionProbeStatus {
    match probe {
        PartitionProbe::Absent => RelationalRegionProbeStatus::Absent,
        PartitionProbe::Cold => RelationalRegionProbeStatus::Cold,
        PartitionProbe::Resident(_) => RelationalRegionProbeStatus::Resident,
    }
}

fn partition_slot_census(
    state: &crate::storage::overlay::PartitionState,
) -> RelationalPartitionSlotCensus {
    let occupied_entity_slots = state.entity_arena.occupied_slots();
    let occupied_entity_generations = occupied_entity_slots
        .iter()
        .filter_map(|&slot| {
            let physical = state.entity_arena.physical_index(slot)?;
            let view = crate::storage::substrate::SlotView::new(&state.entity_arena, physical);
            Some((slot, view.generation()))
        })
        .collect();
    RelationalPartitionSlotCensus {
        entity_slot_count: state.entity_arena.slot_count(),
        occupied_entity_slots,
        occupied_entity_generations,
        relation_slot_count: state.relation_arena.slot_count(),
        occupied_relation_slots: state.relation_arena.occupied_slots(),
    }
}

impl RelationalRuntime {
    pub fn observe_lineage_residency(&self) -> RelationalLineageResidencyObservation {
        let (node_identity_frontier, event_identity_frontier) = self.lineage.identity_frontiers();
        RelationalLineageResidencyObservation {
            resident_node_count: self.lineage.node_count(),
            resident_event_count: self.lineage.event_count(),
            node_identity_frontier,
            event_identity_frontier,
        }
    }
    pub fn has_resident_lineage_node(&self, lineage_id: LineageId) -> bool {
        self.lineage.node(lineage_id).is_some()
    }
    pub fn observe_replay_retention_residency(&self) -> RelationalReplayRetentionObservation {
        RelationalReplayRetentionObservation {
            retained_versions: self.visibility.replay_retention_versions(),
            cached_historical_state_count: self.visibility.recent_visibility_cache_count(),
        }
    }
    pub fn entity_replay_pin_count(&self, entity: EntityId) -> Option<u32> {
        let edition = self.peek_partition_edition();
        let PartitionProbe::Resident(partition) =
            edition.peek_full_region_with_symbols(entity.partition_id, &self.services.symbols)
        else {
            return None;
        };
        partition.entity_arena.replay_pin_count(entity.slot_index())
    }
    pub fn observe_mirror_region_residency(
        &self,
        partition: PartitionId,
    ) -> RelationalRegionResidencyObservation {
        let edition = self.peek_partition_edition();
        RelationalRegionResidencyObservation {
            head: head_status(
                edition.peek_head_region_with_symbols(partition, &self.services.symbols),
            ),
            full: full_status(
                edition.peek_full_region_with_symbols(partition, &self.services.symbols),
            ),
        }
    }
    /// Point-in-time pressure on this runtime's bounded working set, or `None`
    /// when `storage.residency_budget` is unset. Observation grants nothing.
    pub fn residency_pressure(
        &self,
    ) -> Option<crate::storage::residency::working_set::ResidencyPressureSnapshot> {
        self.partitions.residency_pressure()
    }

    pub fn observe_branch_region_residency(
        &self,
        branch: &BranchId,
        partition: PartitionId,
    ) -> Option<RelationalRegionResidencyObservation> {
        let root = self.history.branch_cell(branch)?.root()?;
        let status = |resident| {
            if resident {
                RelationalRegionProbeStatus::Resident
            } else {
                RelationalRegionProbeStatus::Cold
            }
        };
        Some(
            match root.region_residency_census(partition, &self.services.symbols) {
                None => RelationalRegionResidencyObservation {
                    head: RelationalRegionProbeStatus::Absent,
                    full: RelationalRegionProbeStatus::Absent,
                },
                Some((head, full)) => RelationalRegionResidencyObservation {
                    head: status(head),
                    full: status(full),
                },
            },
        )
    }
    pub fn observe_partition_slot_census(
        &self,
        partition: PartitionId,
    ) -> Option<RelationalPartitionSlotCensus> {
        let edition = self.peek_partition_edition();
        let PartitionProbe::Resident(state) =
            edition.peek_full_region_with_symbols(partition, &self.services.symbols)
        else {
            return None;
        };
        Some(partition_slot_census(state))
    }
    pub fn observe_branch_partition_slot_census(
        &self,
        branch: &BranchId,
        partition: PartitionId,
    ) -> Option<RelationalPartitionSlotCensus> {
        let root = self.history.branch_cell(branch)?.root()?;
        let root = observed_read(&root, partition);
        if let PartitionProbe::Resident(state) =
            root.peek_full_region_with_symbols(partition, &self.services.symbols)
        {
            return Some(partition_slot_census(state));
        }
        let PartitionHeadProbe::Resident(head) =
            root.peek_head_region_with_symbols(partition, &self.services.symbols)
        else {
            return None;
        };
        let entities = head.entity_arena();
        let occupied_entity_slots: Vec<usize> = entities.occupied_slots_iter().collect();
        let occupied_entity_generations = occupied_entity_slots
            .iter()
            .filter_map(|&slot| {
                entities
                    .get_slot(slot)
                    .map(|view| (slot, view.generation()))
            })
            .collect();
        let relations = head.relation_arena();
        Some(RelationalPartitionSlotCensus {
            entity_slot_count: entities.slot_count(),
            occupied_entity_slots,
            occupied_entity_generations,
            relation_slot_count: relations.slot_count(),
            occupied_relation_slots: relations.occupied_slots_iter().collect(),
        })
    }
    pub fn observe_resident_symbol_dictionary(&self) -> RelationalResidentSymbolDictionary {
        let snapshot = self.services.symbols.snapshot();
        RelationalResidentSymbolDictionary {
            entries: snapshot
                .entries
                .into_iter()
                .map(|(symbol, text)| (symbol, text.as_str().to_owned()))
                .collect(),
        }
    }
    pub fn observe_resident_entity_metadata_history_len(
        &self,
        branch: &BranchId,
        entity: EntityId,
    ) -> Option<usize> {
        let root = self.history.branch_cell(branch)?.root()?;
        let root = observed_read(&root, entity.partition_id);
        let PartitionProbe::Resident(state) =
            root.peek_full_region_with_symbols(entity.partition_id, &self.services.symbols)
        else {
            return None;
        };
        state
            .entity_arena
            .metadata_history_at(entity.slot_index())
            .map(|column| column.len())
    }
    pub fn observe_complexity_counters(&self) -> RuntimeComplexityCounters {
        self.performance_access().complexity_counters_snapshot()
    }
    pub fn retained_branch_transition_count(&self) -> usize {
        self.history.retained_branch_transitions().1.len()
    }
    pub fn pending_canonical_publication_route_count(&self) -> usize {
        pending_canonical_publication_route_count(&self.history)
    }
    pub fn observe_carried_index_checkpoint_image(&self) -> Option<RelationalDurableMemberLocator> {
        self.indexes.carried_checkpoint_image()
    }
}

/// A census observes what a fresh read admits without storage I/O: a root
/// retains no residency of its own once its readers finish (Store M4 D1).
fn observed_read(
    root: &crate::branch::RelationalBranchRoot,
    partition: PartitionId,
) -> crate::branch::root::RelationalRootRead<'_> {
    let read = root.read();
    let _ = read.probe_full_region(partition);
    let _ = read.probe_head_region(partition);
    read
}
