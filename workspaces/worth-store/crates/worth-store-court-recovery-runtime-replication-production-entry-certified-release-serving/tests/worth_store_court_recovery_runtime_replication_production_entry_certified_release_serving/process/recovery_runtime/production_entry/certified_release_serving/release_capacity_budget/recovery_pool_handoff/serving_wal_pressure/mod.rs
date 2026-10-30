//! A genuine one-shot C8/Store seal must freshly read WAL before Serving.
#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::serving_wal_pressure::*;
use {crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::checkpoint, crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::open_with_policy, crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::recover_core, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};

#[test]
fn sealed_serving_wal_payload_native_pressure_denies_and_fresh_recovery_retries() {
    std::thread::Builder::new()
        .name("sealed-serving-wal-native-pressure".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xe3; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let before_media = snapshot_family(root);
            let payloads = wal_payload_sizes(root);
            let payload_bytes = payloads.iter().map(|(_, bytes)| *bytes).sum::<u64>();
            let core = recover_core(root);
            let policy = core.residency_policy();
            let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(policy.record_format());
            let original = core.recovery_allocation_admission().byte_limit();
            let observer = core.certification_residency_allocations();
            let dimension = worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
            let retained_bytes = observer.snapshot().for_dimension(dimension).active_units();
            // Real file lengths, not a copied native chooser or another policy:
            // bounded bootstrap frame loads have at least one full format page.
            // One byte short of the actual WAL payloads cannot materialize them.
            let available = payload_bytes.checked_sub(1).expect("genuine nonempty WAL");
            assert!(
                available >= u64::from(format.declaration().page_size().bytes()),
                "actual WAL must leave enough capacity for bounded bootstrap loads"
            );
            let peer_bytes = original
                .checked_sub(retained_bytes)
                .unwrap()
                .checked_sub(available)
                .expect("actual WAL fits the original recovery profile");
            let peer = core
                .certification_begin_recovery_allocation(std::num::NonZeroU64::new(peer_bytes).unwrap())
                .unwrap();
            assert_eq!(peer.observation().pool(), observer.snapshot().pool());
            let seal = core
                .certification_release_media_into_checkpoint_custody()
                .expect("genuine one-shot release seal");
            let outcome = open_with_policy(root, seal, format, policy).into_raw();
            let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
                panic!("actual sealed Serving admission must deny before becoming Serving");
            };
            let worth_store::physical_runtime::RecordBootstrapDenial::RecoveredWalRead(failure) = denial.reason() else {
                panic!(
                    "peer must reach WAL freshness, not an earlier bootstrap boundary: {:?}",
                    denial.reason()
                );
            };
            let worth_store::physical_runtime::RecoveryWalReadFailureView::Allocation {
                artifact: worth_store::physical_runtime::RecoveryWalArtifactView::WalArtifact(name),
                offset,
                requested,
                cause:
                    worth_store::physical_runtime::PhysicalRecoveryObservationAllocationDenial::Residency(
                        worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                    ),
            } = failure.diagnostic()
            else {
                panic!("actual sealed Serving WAL payload admission must deny: {failure:?}");
            };
            assert_eq!(offset, 0);
            // The actual segment payload, not a provider envelope, is what the
            // remaining WAL-sized capacity cannot fund.
            assert!(payloads
                .iter()
                .any(|(segment, bytes)| segment.as_os_str() == name && *bytes == requested as u64));
            assert_eq!(*admitted, original);
            assert!(*required > original);
            assert_eq!(snapshot_family(root), before_media);
            let runtime = denial.into_runtime();
            let media = runtime.media_counters();
            for role in [
                worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite,
                worth_store::physical_runtime::certification::MediaOperationRole::Append,
                worth_store::physical_runtime::certification::MediaOperationRole::Truncate,
                worth_store::physical_runtime::certification::MediaOperationRole::Allocate,
                worth_store::physical_runtime::certification::MediaOperationRole::AtomicReplace,
                worth_store::physical_runtime::certification::MediaOperationRole::Delete,
            ] {
                assert_eq!(
                    media.completed_operations_for(role),
                    0,
                    "no Serving effect: {role:?}"
                );
            }
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                peer.bytes() + failure.charged_bytes(),
                "only the actual peer and the failure's retained names remain"
            );
            runtime.close();
            drop(failure);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                peer.bytes()
            );
            drop(peer);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                0
            );
            crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::assert_disposed("sealed Serving native denial disposal", |dimension| {
                let counters = observer.snapshot().for_dimension(dimension);
                (
                    counters.active_units(),
                    counters.admitted_units(),
                    counters.released_units(),
                )
            });
            assert_eq!(snapshot_family(root), before_media);

            // The failed seal is consumed. Retry obtains a new genuine C8/Store
            // seal from unchanged media; it never reuses or reconstructs one.
            let fresh = recover_core(root);
            assert_eq!(fresh.residency_policy(), policy);
            let fresh_observer = fresh.certification_residency_allocations();
            let seal = fresh
                .certification_release_media_into_checkpoint_custody()
                .expect("fresh genuine release seal");
            let worth_proof::TransitionOutcome::Success(serving) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("unchanged policy and media must admit without the actual peer");
            };
            checkpoint(&serving, [0xe4; 32]);
            serving.close();
            crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::assert_disposed("healthy fresh Serving disposal", |dimension| {
                let counters = fresh_observer.snapshot().for_dimension(dimension);
                (
                    counters.active_units(),
                    counters.admitted_units(),
                    counters.released_units(),
                )
            });
            deny_changed_wal_shape(root, format, policy);
        })
        .unwrap()
        .join()
        .expect("genuine sealed Serving pressure worker");
}
