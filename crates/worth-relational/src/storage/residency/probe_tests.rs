use super::*;
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};

#[test]
fn a_cold_round_discards_absence_before_accepting_the_resident_answer() {
    let fixture = region_with_history();
    let edition = crate::runtime::PartitionEdition::new(BTreeMap::from([(
        fixture.entity.partition_id,
        RelationalRegionRetention::new(fixture.cell.clone(), None),
    )]));
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut rounds = 0;
    let answer = probe_full(&execution, &edition, |view| {
        rounds += 1;
        view.get_partition(fixture.entity.partition_id)
            .and_then(|partition| partition.entity_arena.get(&fixture.entity))
            .map(|slot| slot.generation())
    })
    .unwrap();
    assert_eq!(answer, Some(fixture.entity.generation_value()));
    assert!(rounds > 1 && rounds < 4);
    assert!(fixture.source.history_reads.load(Ordering::SeqCst) > 0);
    assert!(fixture.cell.leases.load(Ordering::SeqCst) > 0);
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    let warm = probe_full(&execution, &edition, |view| {
        view.get_partition(fixture.entity.partition_id).is_some()
    })
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
    let edition = crate::runtime::PartitionEdition::new(BTreeMap::from([(
        fixture.entity.partition_id,
        RelationalRegionRetention::new(fixture.cell.clone(), None),
    )]));
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let attempt = || {
        probe_full(&execution, &edition, |view| {
            if view.get_partition(fixture.entity.partition_id).is_none() {
                Err::<(), _>("provisional record missing")
            } else {
                Ok(())
            }
        })
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
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partition_ids_iter().collect()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new((0..self.admitted.len()).map(|id| PartitionId(id as u32)))
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        crate::storage::overlay::collect_partition_ids(
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
fn deep_or_wide_discovery_escalates_and_drops_the_round_before_loading() {
    let fixture = region_with_history();
    let full = full_demand(&fixture.cell, 1 << 24).unwrap();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    for stop_on_first_miss in [true, false] {
        let source = DiscoverySource {
            partition: full.full_view().partition().clone(),
            admitted: (0..128).map(|_| AtomicBool::new(false)).collect(),
            abandoned: Arc::new(AtomicBool::new(false)),
        };
        let mut rounds = 0;
        let answer = probe_full(&execution, &source, |view| {
            rounds += 1;
            let mut complete = true;
            for id in view.partition_ids_iter() {
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
        assert!(
            rounds > 1 && rounds < 32,
            "a long chain must not cause one round per region"
        );
        assert!(source
            .admitted
            .iter()
            .all(|flag| flag.load(Ordering::SeqCst)));
    }
}
