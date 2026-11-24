use super::*;
use contracts::identity::data::primitives::PartitionId;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::residency::probe::probe_full_sources;

#[test]
fn related_sources_do_not_confuse_equal_partition_ids() {
    let fixtures = [region_with_history(), region_with_history()];
    assert_eq!(
        fixtures[0].entity.partition_id,
        fixtures[1].entity.partition_id
    );
    let editions = fixtures.each_ref().map(|fixture| {
        worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition::new(
            [(
                fixture.entity.partition_id,
                RelationalRegionRetention::new(fixture.cell.clone(), None),
            )]
            .into_iter()
            .collect(),
        )
    });
    let symbols =
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable::default();
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let id = fixtures[0].entity.partition_id;
    let mut rounds = 0;
    let answer = probe_full_sources(
        &execution,
        [
            &editions[0].with_symbol_admission(&symbols),
            &editions[1].with_symbol_admission(&symbols),
        ],
        |views| {
            rounds += 1;
            views.map(|view| view.get_partition(id).is_some())
        },
    )
    .unwrap();
    assert_eq!(answer, [true, true]);
    assert!(rounds > 1 && rounds < 4);
    for fixture in fixtures {
        assert!(fixture.source.history_reads.load(Ordering::SeqCst) > 0);
    }
}

#[test]
fn a_resident_overlay_does_not_hide_its_cold_before_image() {
    let base = region_with_history();
    let staged = region_with_history();
    assert_eq!(base.entity.partition_id, staged.entity.partition_id);
    let resident = full_demand(&staged.cell, 1 << 24).unwrap();
    // This is a composite read-view test. It does not assert that a foreign
    // runtime's staged records can acquire mutation or publication authority.
    let staged = BTreeMap::from([(
        base.entity.partition_id,
        resident.full_view().partition().clone(),
    )]);
    let edition =
        worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition::new(
            [(
                base.entity.partition_id,
                RelationalRegionRetention::new(base.cell.clone(), None),
            )]
            .into_iter()
            .collect(),
        );
    let symbols =
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable::default();
    let source = edition.with_symbol_admission(&symbols);
    let overlay = FullCompositeSource {
        base: &source,
        staged: &staged,
        current:
            worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView::new(
                &edition, &staged,
            ),
    };
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut rounds = 0;
    let history = probe_full_sources(&execution, [&overlay], |[view]| {
        rounds += 1;
        assert!(view.get_partition(base.entity.partition_id).is_some());
        view.base_partition(base.entity.partition_id)
            .map(|partition| {
                partition
                    .entity_arena
                    .metadata_history_at(base.entity.slot_index())
                    .unwrap()
                    .len()
            })
    })
    .unwrap();
    assert_eq!(history, Some(base.expected_history));
    assert!(rounds > 1 && rounds < 4);
    assert!(base.source.history_reads.load(Ordering::SeqCst) > 0);
}

#[test]
fn a_cold_round_discards_absence_before_accepting_the_resident_answer() {
    let fixture = region_with_history();
    let edition =
        worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition::new(
            [(
                fixture.entity.partition_id,
                RelationalRegionRetention::new(fixture.cell.clone(), None),
            )]
            .into_iter()
            .collect(),
        );
    let symbols =
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable::default();
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut rounds = 0;
    let answer = probe_full(
        &execution,
        &edition.with_symbol_admission(&symbols),
        |view| {
            rounds += 1;
            view.get_partition(fixture.entity.partition_id)
                .and_then(|partition| partition.entity_arena.get(&fixture.entity))
                .map(|slot| slot.generation())
        },
    )
    .unwrap();
    assert_eq!(answer, Some(fixture.entity.generation_value()));
    assert!(rounds > 1 && rounds < 4);
    assert!(fixture.source.history_reads.load(Ordering::SeqCst) > 0);
    assert!(fixture.cell.leases.load(Ordering::SeqCst) > 0);
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    let warm = probe_full(
        &execution,
        &edition.with_symbol_admission(&symbols),
        |view| view.get_partition(fixture.entity.partition_id).is_some(),
    )
    .unwrap();
    assert!(warm);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
    drop(edition);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
}

#[test]
fn a_provisional_denial_cannot_mask_sticky_storage_damage() {
    let fixture = region_with_history();
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let edition =
        worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition::new(
            [(
                fixture.entity.partition_id,
                RelationalRegionRetention::new(fixture.cell.clone(), None),
            )]
            .into_iter()
            .collect(),
        );
    let symbols =
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable::default();
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let attempt = || {
        probe_full(
            &execution,
            &edition.with_symbol_admission(&symbols),
            |view| {
                if view.get_partition(fixture.entity.partition_id).is_none() {
                    Err::<(), _>("provisional record missing")
                } else {
                    Ok(())
                }
            },
        )
    };
    let first = attempt().unwrap_err();
    assert!(matches!(
        first,
        RelationalResidencyDemandFailure::StorageUnavailable(_)
    ));
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    fixture.source.corrupt.store(false, Ordering::SeqCst);
    assert_eq!(attempt().unwrap_err(), first);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
}

/// This source tests discovery policy, not member readmission. The real cell
/// tests above cover Store-facing admission and retained lease custody.
struct DiscoverySource {
    partition: PartitionState,
    admitted: Vec<AtomicBool>,
    abandoned: Arc<AtomicBool>,
}

impl PartitionAccess for DiscoverySource {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.admitted
            .get(id.0 as usize)
            .filter(|flag| flag.load(Ordering::SeqCst))
            .map(|_| &self.partition)
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        match self.get_partition(id) {
            Some(partition) => PartitionProbe::Resident(partition),
            None => PartitionProbe::Cold,
        }
    }
    fn demand_full_partition(
        &self,
        _: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        assert!(
            self.abandoned.load(Ordering::SeqCst),
            "drop provisional custody before loading"
        );
        self.admitted[id.0 as usize].store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess for DiscoverySource {
    fn get_current_partition(
        &self,
        id: contracts::identity::data::primitives::PartitionId,
    ) -> Option<worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView<'_>> {
        worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess::get_partition(self, id)
            .map(worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView::from_owned_partition)
    }
    fn probe_head_partition(
        &self,
        id: contracts::identity::data::primitives::PartitionId,
    ) -> worth_relational_storage::internal_port::storage::residency::head_probe::PartitionHeadProbe<'_> {
        use crate::storage::residency::{PartitionHeadProbe as Head, PartitionProbe as Full};
        match worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess::probe_full_partition(self, id) {
            Full::Absent => Head::Absent,
            Full::Cold => Head::Cold,
            Full::Resident(partition) => Head::Resident(
                worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView::from_owned_partition(
                    partition,
                ),
            ),
        }
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: contracts::identity::data::primitives::PartitionId,
    ) -> Result<(), worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure> {
        worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess::demand_full_partition(self, execution, id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partition_ids_iter().collect()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new((0..self.admitted.len()).map(|id| PartitionId(id as u32)))
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn worth_relational_storage::internal_port::storage::overlay::access::memory::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, worth_relational_storage::internal_port::storage::overlay::access::memory::PartitionIdsDenial> {
        worth_relational_storage::internal_port::storage::overlay::access::memory::collect_partition_ids(
            self.admitted.len(),
            self.partition_ids_iter(),
            memory,
        )
    }
}

struct DiscoveryAnswer {
    complete: bool,
    abandoned: Arc<AtomicBool>,
}

impl Drop for DiscoveryAnswer {
    fn drop(&mut self) {
        if !self.complete {
            self.abandoned.store(true, Ordering::SeqCst);
        }
    }
}

#[test]
fn deep_or_wide_discovery_loads_only_observed_misses_after_dropping_the_round() {
    let fixture = region_with_history();
    let full = full_demand(&fixture.cell, 1 << 24).unwrap();
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    for stop_on_first_miss in [true, false] {
        let source = DiscoverySource {
            partition: full.full_view().partition().clone(),
            admitted: (0..128).map(|_| AtomicBool::new(false)).collect(),
            abandoned: Arc::new(AtomicBool::new(false)),
        };
        let answer = probe_full(&execution, &source, |view| {
            let mut complete = true;
            for id in (0..64).map(PartitionId) {
                if view.get_partition(id).is_none() {
                    complete = false;
                    if stop_on_first_miss {
                        break;
                    }
                }
            }
            DiscoveryAnswer {
                complete,
                abandoned: source.abandoned.clone(),
            }
        })
        .unwrap();
        assert!(answer.complete);
        assert!(source
            .admitted
            .iter()
            .enumerate()
            .all(|(id, flag)| flag.load(Ordering::SeqCst) == (id < 64)));
    }
}

/// A Full composite over actual Full-owned staged contents and the real cold
/// edition. Preparation overlays deliberately expose only current reads.
struct FullCompositeSource<'source> {
    base: &'source dyn PartitionAccess,
    staged: &'source BTreeMap<PartitionId, PartitionState>,
    current: worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView<
        'source,
        BTreeMap<PartitionId, PartitionState>,
    >,
}
impl PartitionAccess for FullCompositeSource<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.staged.get(&id).or_else(|| self.base.get_partition(id))
    }
    fn base_source(&self) -> Option<&dyn PartitionAccess> {
        Some(self.base)
    }
    fn base_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.base.get_partition(id)
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        match self.staged.get(&id) {
            Some(partition) => PartitionProbe::Resident(partition),
            None => self.base.probe_full_partition(id),
        }
    }
    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        if self.staged.contains_key(&id) {
            return contracts::execution::request_custody::check_request(execution)
                .map_err(RelationalResidencyDemandFailure::Request);
        }
        self.base.demand_full_partition(execution, id)
    }
}
impl CurrentPartitionAccess for FullCompositeSource<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.current.get_current_partition(id)
    }
    fn base_current_source(&self) -> Option<&dyn CurrentPartitionAccess> {
        Some(self.base)
    }
    fn base_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.current.base_current_partition(id)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.current.partition_ids()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn worth_relational_storage::internal_port::storage::overlay::access::memory::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, worth_relational_storage::internal_port::storage::overlay::access::memory::PartitionIdsDenial>{
        self.current.partition_ids_with_memory(memory)
    }
}
