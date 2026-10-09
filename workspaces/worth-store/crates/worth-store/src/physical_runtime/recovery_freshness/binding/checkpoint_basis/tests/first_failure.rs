//! The first semantic failure stops decode work, not exact C9 aggregation.

use super::*;
use crate::physical_runtime::durability::with_conflicting_binding_history;

#[test]
fn first_binding_conflict_prevents_later_native_decode_and_preserves_prefix_census() {
    with_conflicting_binding_history(|root, source, payloads| {
        let (media, mut coordination) = fixture::reopen(root);
        assert_eq!(media.store_identity(), source.identity().store_identity());
        let observer = coordination.certification_residency_allocations();
        let dimension = Dimension::OperationScope(Scope::Recovery);
        let payload_refs: Vec<_> = payloads.iter().map(Vec::as_slice).collect();
        fixture::with_bindings(source, &payload_refs, |assembly, bindings, records| {
            let shared = coordination.admit_shared_checkpoint(assembly).unwrap();
            let facts = shared.facts();
            assert_eq!(facts.footer().binding_record_count(), 3);
            let mut window = coordination.begin_source_read_allocation().unwrap();
            let mut poisoned = window.begin_checkpoint_binding_rebuild(&shared, 3).unwrap();
            let mut healthy = window.begin_checkpoint_binding_rebuild(&shared, 3).unwrap();
            poisoned
                .consume(&bindings[0], records[0], &mut window)
                .unwrap();
            let first = observer.snapshot().for_dimension(dimension);
            poisoned
                .consume(&bindings[1], records[1], &mut window)
                .unwrap();
            let second = observer.snapshot().for_dimension(dimension);
            assert_eq!(
                second.admissions(),
                first.admissions() + 1,
                "the conflicting binding actually reaches semantic decode"
            );
            let failure = poisoned.failure.as_ref().unwrap().clone();
            assert_eq!(
                failure.denial(),
                StoreRecoveryBindingSampleDenial::ConflictingOperationEvidence
            );
            assert_eq!(failure.operation_bindings_observed(), 1);
            assert_eq!(failure.freshness_retained(), 1);
            assert_eq!(failure.freshness_expired(), 0);
            drop(window);

            // Both builders are independently funded before the actual blocker.
            // A healthy twin consumes the same heap-bearing third C9 record to
            // prove this pressure would deny decoding, rather than assume it.
            let requested = checkpoint_binding_decode_peak(payloads[2].len()).unwrap();
            let active = observer.snapshot().for_dimension(dimension).active_units();
            let held_bytes = ORIGINAL
                .checked_sub(active)
                .unwrap()
                .checked_sub(requested)
                .unwrap()
                + 1;
            let held = coordination
                .certification_begin_recovery_allocation(NonZeroU64::new(held_bytes).unwrap())
                .unwrap();
            let mut window = coordination.begin_source_read_allocation().unwrap();
            let before_twin = observer.snapshot().for_dimension(dimension);
            assert_eq!(
                healthy.consume(&bindings[2], records[2], &mut window),
                Err(Denial::Backing {
                    requested,
                    cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                        required: ORIGINAL + 1,
                        admitted: ORIGINAL,
                    }
                })
            );
            let after_twin = observer.snapshot().for_dimension(dimension);
            assert_eq!(after_twin.admissions(), before_twin.admissions());
            assert_eq!(after_twin.admitted_units(), before_twin.admitted_units());
            assert_eq!(after_twin.denials(), before_twin.denials() + 1);
            assert_eq!(window.charged_bytes(), 0);

            let before_stopped = observer.snapshot();
            poisoned
                .consume(&bindings[2], records[2], &mut window)
                .unwrap();
            assert_eq!(
                observer.snapshot(),
                before_stopped,
                "an existing conflict forbids any further native decode admission or denial"
            );
            let summary = poisoned.aggregate.summary();
            assert_eq!(
                summary.record_count(),
                facts.footer().binding_record_count()
            );
            assert_eq!(
                summary.encoded_bytes(),
                facts.footer().binding_record_bytes()
            );
            assert_eq!(summary.digest(), facts.footer().binding_records_digest());
            let basis = poisoned.finish().unwrap();
            assert!(basis.matches_checkpoint(&facts));
            let finished_failure = basis.outcome.as_ref().unwrap_err();
            assert_eq!(
                finished_failure, &failure,
                "finish preserves the intended first failure and its one-binding prefix census"
            );
            assert_eq!(finished_failure.wal_members_observed(), 0);
            assert_eq!(finished_failure.redo_bytes_observed(), 0);
            assert_eq!(basis.owned_heap_bytes(), Some(0));
            assert_eq!(basis.charged_bytes(), 0);
            drop(basis);
            drop(healthy);
            drop(held);
            drop(window);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                shared.owned_heap_bytes().unwrap()
            );
            drop(shared);
        });
        drop(coordination);
        drop(media);
        let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
        assert_eq!(disposed.active_units(), 0);
        assert_eq!(disposed.admitted_units(), disposed.released_units());
    });
}
