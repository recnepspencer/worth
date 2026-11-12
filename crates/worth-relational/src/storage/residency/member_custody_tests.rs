//! Placement owners survive the cell when a mutation suffix retains its base.
use super::region_fixture::{with_memory_request, CustodySource};
use super::*;
use crate::durability::medium::RelationalDurableMemberCustody;
use crate::storage::overlay::PartitionCloneMode;
use std::sync::Mutex;

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
    cold.source = Some(source);
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
    cold.source = Some(source.clone());
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

#[test]
fn controlled_custody_refuses_original_memory_before_registration() {
    use crate::durability::medium::RelationalDurableOwnedReadRefusal;
    use worth_execution::SerialMemoryBudget;
    let fixture = region_with_history();
    let active = Arc::new(AtomicUsize::new(0));
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(CustodySource {
        source: fixture.source,
        active: active.clone(),
        refusal: Mutex::new(None),
    });
    let budget = SerialMemoryBudget::new(Some(0));
    with_memory_request(budget.clone(), |request| {
        assert!(matches!(
            RelationalDurableMemberCustody::capture(&source, &request, &[fixture.cold.history]),
            Err(RelationalDurableOwnedReadRefusal::Request(
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            ))
        ));
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(budget.charged_memory_bytes(), 0);
    });
}

#[test]
fn controlled_custody_disarms_its_last_registration_before_inert_payload_reclamation() {
    use crate::durability::medium::RelationalDurableMemberRetirementObservation as Observation;
    use worth_execution::SerialMemoryBudget;
    let fixture = region_with_history();
    let active = Arc::new(AtomicUsize::new(0));
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(CustodySource {
        source: fixture.source,
        active: active.clone(),
        refusal: Mutex::new(None),
    });
    let budget = SerialMemoryBudget::new(None);
    with_memory_request(budget.clone(), |request| {
        let token =
            RelationalDurableMemberCustody::capture(&source, &request, &[fixture.cold.history])
                .unwrap();
        assert_eq!(active.load(Ordering::SeqCst), 1);
        assert!(budget.charged_memory_bytes() > 0);
        let last = token.clone();
        let mut share = token.into_retirement();
        assert!(matches!(
            share.advance(&request, source.as_ref()),
            Observation::Complete
        ));
        share.into_retired().unwrap().reclaim();
        assert_eq!(
            active.load(Ordering::SeqCst),
            1,
            "another real share retains the registration"
        );
        let mut final_share = last.into_retirement();
        // Bound traversal of outer admission, child admission and actual release.
        for _step in 0..8 {
            match final_share.advance(&request, source.as_ref()) {
                Observation::Advanced => {}
                Observation::Complete => break,
                Observation::Busy | Observation::Stopped(_) | Observation::Refused(_) => {
                    panic!("the admitted immutable child must retire without another source")
                }
            }
        }
        assert_eq!(
            active.load(Ordering::SeqCst),
            0,
            "release is explicit, not payload Drop"
        );
        assert!(
            budget.charged_memory_bytes() > 0,
            "the inert owner still retains its backing grant"
        );
        final_share.into_retired().unwrap().reclaim();
        assert_eq!(budget.charged_memory_bytes(), 0);
    });
}
