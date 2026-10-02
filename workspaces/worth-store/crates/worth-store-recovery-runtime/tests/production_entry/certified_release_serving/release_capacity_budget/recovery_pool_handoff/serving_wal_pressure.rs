//! A genuine one-shot C8/Store seal must freshly read WAL before Serving.
use super::*;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store::physical_runtime::{
    ArtifactTreeListingAllocationBoundary, PhysicalRecoveryObservationAllocationDenial,
    RecordBootstrapDenial, RecoveryWalArtifactView, RecoveryWalReadFailureView,
};

#[test]
fn sealed_serving_wal_provider_native_pressure_denies_and_fresh_recovery_retries() {
    std::thread::Builder::new()
        .name("sealed-serving-wal-native-pressure".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
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
            let format = AdmittedPhysicalRecordFormat::admit(policy.record_format());
            let original = core.recovery_allocation_admission().byte_limit();
            let observer = core.certification_residency_allocations();
            let dimension = Dimension::OperationScope(Scope::Recovery);
            let retained_bytes = observer.snapshot().for_dimension(dimension).active_units();
            // Real file lengths, not a copied native chooser or another policy:
            // bounded bootstrap frame loads have at least one full format page.
            // This fixture's actual WAL-sized remainder cannot fund the pinned
            // provider's fixed path-conversion peak before WAL materialization.
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
                .certification_begin_recovery_allocation(NonZeroU64::new(peer_bytes).unwrap())
                .unwrap();
            assert_eq!(peer.observation().pool(), observer.snapshot().pool());
            let seal = core
                .into_checkpoint_custody()
                .expect("genuine one-shot release seal");
            let outcome = open_with_policy(root, seal, format, policy).into_raw();
            let TransitionOutcome::Denied(denial) = outcome else {
                panic!("actual sealed Serving admission must deny before becoming Serving");
            };
            let RecordBootstrapDenial::RecoveredWalRead(failure) = denial.reason() else {
                panic!(
                    "peer must reach WAL freshness, not an earlier bootstrap boundary: {:?}",
                    denial.reason()
                );
            };
            let RecoveryWalReadFailureView::Allocation {
                artifact: RecoveryWalArtifactView::WalDirectory,
                offset,
                requested,
                cause:
                    PhysicalRecoveryObservationAllocationDenial::ListingResidency {
                        boundary: ArtifactTreeListingAllocationBoundary::ProviderPath,
                        cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                    },
            } = failure.diagnostic()
            else {
                panic!(
                    "actual sealed Serving WAL provider admission must deny: {failure:?}"
                );
            };
            assert_eq!(offset, 0);
            // Qualified Windows provider contract: winx's fixed 0x7fff UTF-16
            // path buffer plus simultaneous WTF-8 conversion growth.
            assert_eq!(requested, 14 * 0x7fff);
            assert!(requested as u64 > available);
            assert_eq!(*admitted, original);
            assert!(*required > original);
            assert_eq!(snapshot_family(root), before_media);
            let runtime = denial.into_runtime();
            let media = runtime.media_counters();
            for role in [
                MediaOperationRole::PositionedWrite,
                MediaOperationRole::Append,
                MediaOperationRole::Truncate,
                MediaOperationRole::Allocate,
                MediaOperationRole::AtomicReplace,
                MediaOperationRole::Delete,
            ] {
                assert_eq!(
                    media.completed_operations_for(role),
                    0,
                    "no Serving effect: {role:?}"
                );
            }
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                peer.bytes(),
                "only the actual peer remains after inline provider denial"
            );
            assert_eq!(failure.charged_bytes(), 0, "directory denial has no copied filename owner");
            runtime.close();
            drop(failure);
            assert_eq!(observer.snapshot().for_dimension(dimension).active_units(), peer.bytes());
            drop(peer);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                0
            );
            backing_census::assert_disposed("sealed Serving native denial disposal", |dimension| {
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
                .into_checkpoint_custody()
                .expect("fresh genuine release seal");
            let TransitionOutcome::Success(serving) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("unchanged policy and media must admit without the actual peer");
            };
            checkpoint(&serving, [0xe4; 32]);
            serving.close();
            backing_census::assert_disposed("healthy fresh Serving disposal", |dimension| {
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

fn deny_changed_wal_shape(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) {
    for extra_file in [false, true] {
        let before = snapshot_family(root);
        let core = recover_core(root);
        let observer = core.certification_residency_allocations();
        let seal = core
            .into_checkpoint_custody()
            .expect("fresh genuine seal before external mutation");
        let (name, _) = wal_payload_sizes(root).into_iter().next().unwrap();
        let path = root.join("families/wal").join(name);
        let original = fs::read(&path).unwrap();
        let extra = root.join("families/wal/unrecognized-tail.wal");
        if extra_file {
            assert!(!extra.exists());
            fs::write(&extra, b"unknown actual directory entry").unwrap();
        } else {
            let mut enlarged = original.clone();
            enlarged.push(0x57);
            fs::write(&path, enlarged).unwrap();
        }
        let altered = snapshot_family(root);
        let TransitionOutcome::Denied(denial) =
            open_with_policy(root, seal, format, policy).into_raw()
        else {
            panic!("actual post-seal WAL shape change must not become Serving");
        };
        assert_eq!(
            denial.reason(),
            RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
        );
        let runtime = denial.into_runtime();
        assert_eq!(runtime.media_counters().replacements(), 0);
        assert_eq!(runtime.media_counters().deletions(), 0);
        assert_eq!(
            runtime
                .media_counters()
                .completed_operations_for(MediaOperationRole::PositionedWrite),
            0
        );
        assert_eq!(
            runtime
                .media_counters()
                .completed_operations_for(MediaOperationRole::Append),
            0
        );
        assert_eq!(
            snapshot_family(root),
            altered,
            "only the deliberate external mutation exists"
        );
        runtime.close();
        backing_census::assert_disposed("post-seal shape mismatch disposal", |dimension| {
            let counters = observer.snapshot().for_dimension(dimension);
            (
                counters.active_units(),
                counters.admitted_units(),
                counters.released_units(),
            )
        });
        if extra_file {
            fs::remove_file(&extra).unwrap();
        } else {
            fs::write(&path, &original).unwrap();
        }
        assert_eq!(snapshot_family(root), before);
    }
}

fn wal_payload_sizes(root: &Path) -> Vec<(std::ffi::OsString, u64)> {
    let mut payloads = Vec::new();
    for entry in fs::read_dir(root.join("families/wal")).expect("actual retained WAL directory") {
        let entry = entry.unwrap();
        assert!(
            entry.file_type().unwrap().is_file(),
            "genuine WAL fixture has regular segment files"
        );
        payloads.push((entry.file_name(), entry.metadata().unwrap().len()));
    }
    assert!(!payloads.is_empty());
    payloads
}
