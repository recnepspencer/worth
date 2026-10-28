//! Placement owners survive the cell when a mutation suffix retains its base.
use super::*;
use crate::durability::medium::RelationalDurableMemberCustody;
use crate::storage::overlay::PartitionCloneMode;
use std::sync::Mutex;

#[derive(Debug)]
struct CustodySource {
    source: Arc<CountedSource>,
    active: Arc<AtomicUsize>,
    refusal: Mutex<Option<Cause>>,
}

struct CustodyOwner {
    active: Arc<AtomicUsize>,
    _source: RelationalDurableMemberCustody,
}

impl Drop for CustodyOwner {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

impl RelationalDurableColdSource for CustodySource {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if let Some(cause) = *self.refusal.lock().unwrap() {
            return Err(cause);
        }
        let source = self.source.retain_members(members)?;
        self.active.fetch_add(1, Ordering::SeqCst);
        Ok(RelationalDurableMemberCustody::new(CustodyOwner {
            active: self.active.clone(),
            _source: source,
        }))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.source.read_member(locator, limit)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.source.read_commit(position, limit)
    }
}

#[test]
fn an_adopted_suffix_keeps_placement_custody_after_its_source_cell_is_released() {
    let fixture = region_with_history();
    let active = Arc::new(AtomicUsize::new(0));
    let source = Arc::new(CustodySource {
        source: fixture.source.clone(),
        active: active.clone(),
        refusal: Mutex::new(None),
    });
    let mut cold = fixture.cold;
    cold.source = source;
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    assert!(active.load(Ordering::SeqCst) > 0);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 0);
    let head = head_demand(&cell, 1 << 24).unwrap();
    let selected = std::collections::BTreeSet::from([fixture.entity.slot_index()]);
    let suffix = head.head_view().clone_for_mutation(
        PartitionCloneMode::GraphSparseEntities,
        Some(&selected),
        false,
    );
    let adopted = suffix
        .entity_arena
        .sparse_clone_slots_for_overlay(&selected);
    drop(head);
    drop(cell);
    drop(suffix);
    assert!(
        active.load(Ordering::SeqCst) > 0,
        "the adopted prefix still needs its source placement"
    );
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    drop(adopted);
    assert_eq!(active.load(Ordering::SeqCst), 0);
}

#[test]
fn cold_cell_custody_refusal_preserves_the_region_failure_and_is_retryable_without_reads() {
    let fixture = region_with_history();
    let active = Arc::new(AtomicUsize::new(0));
    let source = Arc::new(CustodySource {
        source: fixture.source.clone(),
        active: active.clone(),
        refusal: Mutex::new(Some(Cause::SourceUnavailable)),
    });
    let mut cold = fixture.cold;
    cold.source = source.clone();
    let failure = RelationalRegionResidency::cold(cold.clone()).unwrap_err();
    assert_eq!(
        failure.subject,
        RelationalStorageSubject::Region(fixture.entity.partition_id)
    );
    assert_eq!(failure.cause, Cause::SourceUnavailable);
    let crate::durability::data::RelationalRecoveryStop::Durability(stop) = failure.into() else {
        panic!("placement refusal is a typed cold storage recovery stop");
    };
    assert_eq!(
        stop.class,
        crate::durability::data::RecoveryFailureClass::ColdStorageUnavailable
    );
    assert_eq!(active.load(Ordering::SeqCst), 0);
    *source.refusal.lock().unwrap() = None;
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    assert!(active.load(Ordering::SeqCst) > 0);
    drop(cell);
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 0);
}
