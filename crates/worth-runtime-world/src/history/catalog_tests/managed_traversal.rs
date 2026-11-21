use super::fixtures::{history_contract, linear_history};
use crate::history::{CompositeHistoryCatalog, CompositeHistoryReclamationRequest};
use std::num::NonZeroUsize;
#[test]
fn bounded_traversal_protects_chain_and_reuses_exact_history_leases() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (fixture, commits) = linear_history(3);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(3, u64::MAX));
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    // Absence traverses the genuine retained chain, without a numeric sentinel.
    let full = catalog
        .trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution,
                maximum: None,
            },
            commits[2].identity().clone(),
            None,
        )
        .unwrap();
    assert_eq!(full.visited_count(), 3);
    assert!(full.is_complete());
    assert_eq!(
        full.commits().last().unwrap().identity(),
        commits[0].identity()
    );
    drop(full);
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let held = fixture.retention.active_component_obligation_count();
    assert!(matches!(
        catalog.trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution: worth_execution::ExecutionRequest::serial(&stopped),
                maximum: None
            },
            commits[2].identity().clone(),
            None
        ),
        Err(crate::history::CompositeHistoryCatalogDenial::Cancelled)
    ));
    assert_eq!(fixture.retention.active_component_obligation_count(), held);
    let full = catalog
        .trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution,
                maximum: None,
            },
            commits[2].identity().clone(),
            None,
        )
        .unwrap();
    assert_eq!(
        full.visited_count(),
        3,
        "fresh original request retains the same native history"
    );
    drop(full);
    let before = fixture.retention.cost_snapshot();
    let traversal = catalog
        .trace_ancestry(commits[2].identity().clone(), NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(traversal.visited_count(), 1);
    assert_eq!(traversal.next_parent(), Some(commits[1].identity()));
    assert_eq!(
        traversal.commits().next().unwrap().identity(),
        commits[2].identity()
    );
    let request =
        || CompositeHistoryReclamationRequest::new(owner, vec![commits[2].identity().clone()], 1);
    assert_eq!(
        catalog
            .reclaim_batch(request())
            .unwrap()
            .skipped_protected(),
        1
    );
    assert_eq!(fixture.retention.unique_pin_count(), 2);
    assert_eq!(fixture.retention.active_component_obligation_count(), 6);
    assert_eq!(
        fixture
            .retention
            .cost_snapshot()
            .owner_acquisition_contacts(),
        before.owner_acquisition_contacts()
    );
    drop(traversal);
    assert_eq!(
        catalog
            .reclaim_batch(request())
            .unwrap()
            .reclaimed_commits()
            .len(),
        1
    );
    assert_eq!(fixture.retention.active_component_obligation_count(), 4);
    let traversal = catalog
        .trace_ancestry(commits[1].identity().clone(), NonZeroUsize::new(2).unwrap())
        .unwrap();
    drop(catalog);
    assert_eq!(traversal.visited_count(), 2);
    assert_eq!(fixture.retention.active_component_obligation_count(), 4);
    drop(traversal);
    assert_eq!(fixture.retention.active_component_obligation_count(), 0);
}

#[test]
fn overlapping_memory_original_reservations_refund_unneeded_growth() {
    use crate::branch::reference_test_fixture;
    use crate::history::publication::memory_observation::{
        MemoryPublicationMeaning, MemoryPublicationReceiver, MemoryPublicationRecord,
    };
    use std::sync::Arc;
    use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
    use worth_foundational::facade::{
        MemoryProductBranchReference, RecordedCompositePublicationAttemptIdentity,
    };

    let (_hist_fixture, commits) = linear_history(1);
    let owner = commits[0].identity().owner_identity();
    let host = crate::test_execution_policy::serial_request();
    let exec = ExecutionRequest::serial(&host);
    let fixture = reference_test_fixture::real_fixture(4, 4);
    let (_, basis) = fixture
        .relational_runtime()
        .observe_branch(&fixture.relational_runtime().main_branch_identity())
        .unwrap();
    let component = fixture
        .relational_runtime()
        .owner_component_services()
        .basis_port()
        .observe_memory_component(&exec, &basis)
        .unwrap();
    let meaning = Arc::new(MemoryPublicationMeaning {
        before: MemoryProductBranchReference::from_observed(owner.get(), "main".into(), 1, 1, 1)
            .unwrap(),
        after: MemoryProductBranchReference::from_observed(owner.get(), "main".into(), 1, 2, 2)
            .unwrap(),
        attempt: RecordedCompositePublicationAttemptIdentity::Memory {
            owner: owner.get(),
            ordinal: 1,
        },
        _funding: None,
    });
    let make_record = |committed: bool| {
        Arc::new(MemoryPublicationRecord {
            meaning: Arc::clone(&meaning),
            receiver: MemoryPublicationReceiver::Live,
            before_component: component.clone(),
            after_component: component.clone(),
            committed: std::sync::atomic::AtomicBool::new(committed),
            _funding: None,
        })
    };
    let slot_bytes = |slots: usize| {
        (slots * CompositeHistoryCatalog::memory_original_slot_size_for_test()) as u64
    };
    let budgeted = || {
        let budget = SerialMemoryBudget::new(None);
        let req = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
        (budget, req)
    };

    // Schedule 1: two committed originals in capacity 3; A reserves unfunded,
    // B and C reserve funded (7); A drops, B seals (must refund 7-slot growth), C drops.
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(16, 65536));
    let committed = make_record(true);
    let (b1, r1) = budgeted();
    let _reg1 = catalog
        .reserve_memory_original(&ExecutionRequest::serial(&r1))
        .unwrap()
        .fill(Arc::clone(&committed));
    assert_eq!(b1.charged_memory_bytes(), slot_bytes(1));
    let (b2, r2) = budgeted();
    let _reg2 = catalog
        .reserve_memory_original(&ExecutionRequest::serial(&r2))
        .unwrap()
        .fill(Arc::clone(&committed));
    assert_eq!(b2.charged_memory_bytes(), slot_bytes(3));
    assert_eq!(b1.charged_memory_bytes(), 0);

    let (ba, ra) = budgeted();
    let slot_a = catalog
        .reserve_memory_original(&ExecutionRequest::serial(&ra))
        .unwrap();
    assert_eq!(ba.charged_memory_bytes(), 0);
    let (bb, rb) = budgeted();
    let slot_b = catalog
        .reserve_memory_original(&ExecutionRequest::serial(&rb))
        .unwrap();
    assert_eq!(bb.charged_memory_bytes(), slot_bytes(7));
    let (bc, rc) = budgeted();
    let slot_c = catalog
        .reserve_memory_original(&ExecutionRequest::serial(&rc))
        .unwrap();
    assert_eq!(bc.charged_memory_bytes(), slot_bytes(7));

    drop(slot_a);
    let _reg_b = slot_b.fill(Arc::clone(&committed));
    assert_eq!(bb.charged_memory_bytes(), 0);
    assert_eq!(catalog.memory_original_capacity_for_test(), 3);
    drop(slot_c);
    assert_eq!(bc.charged_memory_bytes(), 0);
    assert_eq!(b2.charged_memory_bytes(), slot_bytes(3));

    // Schedule 2: intermediate tier pressure requires adopting the larger
    // backing so remaining smaller-tier reservations can all seal without overflow.
    let catalog2 = CompositeHistoryCatalog::new(owner, history_contract(16, 65536));
    let (_b0, r0) = budgeted();
    let reg0 = catalog2
        .reserve_memory_original(&ExecutionRequest::serial(&r0))
        .unwrap()
        .fill(make_record(false));
    let (_b3, r3) = budgeted();
    let slot_t2_a = catalog2
        .reserve_memory_original(&ExecutionRequest::serial(&r3))
        .unwrap();
    let (_b4, r4) = budgeted();
    let slot_t2_b = catalog2
        .reserve_memory_original(&ExecutionRequest::serial(&r4))
        .unwrap();
    drop(reg0);
    let (_b5, r5) = budgeted();
    let slot_t2_c = catalog2
        .reserve_memory_original(&ExecutionRequest::serial(&r5))
        .unwrap();
    let (b6, r6) = budgeted();
    let slot_t3 = catalog2
        .reserve_memory_original(&ExecutionRequest::serial(&r6))
        .unwrap();
    assert_eq!(b6.charged_memory_bytes(), slot_bytes(7));

    let _reg_t3 = slot_t3.fill(Arc::clone(&committed));
    assert_eq!(catalog2.memory_original_capacity_for_test(), 7);
    assert_eq!(b6.charged_memory_bytes(), slot_bytes(7));
    let _reg_a = slot_t2_a.fill(Arc::clone(&committed));
    let _reg_b = slot_t2_b.fill(Arc::clone(&committed));
    let _reg_c = slot_t2_c.fill(Arc::clone(&committed));
    assert_eq!(catalog2.memory_original_capacity_for_test(), 7);
}
