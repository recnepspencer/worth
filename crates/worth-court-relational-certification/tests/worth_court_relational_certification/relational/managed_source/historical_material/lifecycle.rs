//! Actual opaque destruction and over-aligned weak backing at real Native history.
use super::*;
use std::alloc::Layout;
use worth_relational::facade::mvcc::PublicationCompanionRegistrationPort;

struct Probe {
    drops: Arc<AtomicUsize>,
    port: PublicationCompanionRegistrationPort,
}
impl Drop for Probe {
    fn drop(&mut self) {
        // Reenter the actual registration owner after the cache retirement was
        // handed out. Its live epoch may refuse, but no coordination gate hangs.
        let _ = self.port.begin_required_registration();
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[repr(align(128))]
struct AlignedProbe {
    _owned: Probe,
}

fn retained_quote(aligned: bool) -> u64 {
    let mut retained = 0;
    run_court(
        HistoryProfile::Membership,
        |runtime, program, execution, court, host| {
            let a = create(runtime, execution, 1, "A");
            let x = create(runtime, execution, 4, "x");
            let y = create(runtime, execution, 4, "y");
            let links = commit(
                runtime,
                &execution,
                WorkerIntentBatch::new("weak-A-x").push(edge(a, x, "A-x")),
            );
            let relation = edge_id(&links, a, x, "A-x");
            let before = basis(runtime);
            let original = observe(runtime, execution, program, a);
            let drops = Arc::new(AtomicUsize::new(0));
            let probe = Probe {
                drops: drops.clone(),
                port: runtime.publication_companion_port(),
            };
            let value: Arc<dyn std::any::Any + Send + Sync> = if aligned {
                Arc::new(AlignedProbe { _owned: probe })
            } else {
                Arc::new(probe)
            };
            court.opaque_roots.lock().unwrap().insert(a, value);
            let moved = endpoints(runtime, execution, relation, a, y);
            let change = occurrence(runtime, execution, program, a, &before);
            let Impact::Retained(material) = original
                .authored_history_impact(runtime, execution, &change)
                .unwrap()
            else {
                panic!("actual retained material receipts");
            };
            let shares = material.acquire_payloads(execution).unwrap().unwrap();
            if aligned {
                assert!(shares.after().is::<AlignedProbe>());
            } else {
                assert!(shares.after().is::<Probe>());
            }

            court.opaque_roots.lock().unwrap().remove(&a);
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis(runtime).observation())
                .unwrap();
            let retirement = runtime
                .read_truth()
                .evict_managed_index_source(&execution, &snapshot, program, a)
                .unwrap();
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
            runtime
                .snapshots()
                .release_snapshot(&moved.snapshot)
                .unwrap();
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            drop(retirement); // Actual returned opaque custody leaves Native gates.
            assert_eq!(
                drops.load(Ordering::SeqCst),
                0,
                "the acquired carrier is now the final strong owner"
            );
            drop(shares); // Final opaque disposal is outside every Native/runtime/attempt call.
            assert_eq!(
                drops.load(Ordering::SeqCst),
                1,
                "canonical history and its metadata view must not own opaque destruction"
            );
            assert!(
                material.acquire_payloads(execution).unwrap().is_none(),
                "expired data is Conservative, never empty membership"
            );
            // All other live Native state and event widths are identical between
            // these two real journeys. Only the weak outer payload allocation varies.
            retained = host.memory().unwrap().charged_memory_bytes();
            assert_eq!(material.row_count(), 2);
            assert_eq!(change.commit_id(), moved.commit.commit_id);
        },
    );
    retained
}
fn allocation_bytes(value: Layout) -> u64 {
    // Independent arithmetic oracle for the outer Arc's inline allocation;
    // recursively freed payload heaps are deliberately absent.
    let header = 2 * size_of::<usize>();
    let data = header.div_ceil(value.align()) * value.align();
    let align = value.align().max(align_of::<usize>());
    (data + value.size()).div_ceil(align) as u64 * align as u64
}
#[test]
fn historical_receipts_do_not_retain_opaque_drop_and_fund_expired_overaligned_backing() {
    let plain = retained_quote(false);
    let aligned = retained_quote(true);
    let difference =
        allocation_bytes(Layout::new::<AlignedProbe>()) - allocation_bytes(Layout::new::<Probe>());
    assert_eq!(
        aligned.checked_sub(plain),
        Some(difference),
        "the actual dynamic outer allocation stays funded after opaque destruction"
    );
}
