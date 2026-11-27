use super::*;
use crate::physical_runtime::durability::mutation::idempotency::PhysicalMutationIdempotencyRuntimeOwner;

#[test]
fn concurrent_admission_during_sync_and_saturated_reopen_preserve_cadence_counts() {
    let (mut f, mat) = (fixture(16), PhysicalMutationIdempotencyMaterial::new);
    let (k1, k2) = (
        f.registry.issue_key(mat([51; 32])).unwrap(),
        f.registry.issue_key(mat([52; 32])).unwrap(),
    );
    f.registry
        .admit_unallocated(k1, fingerprint(&f, 1), mutation(&f, 1))
        .unwrap();
    let (fp2, m2) = (fingerprint(&f, 2), mutation(&f, 2));
    let checkpoint =
        PhysicalCheckpointIdentity::new(f.store, std::num::NonZeroU64::new(1).unwrap());
    let owner = PhysicalMutationIdempotencyRuntimeOwner::from_rebuilt_registry(f.registry);
    let auth = PhysicalMutationIdempotencyRuntimeOwner::authority(&owner);
    let comp = PhysicalMutationIdempotencyRuntimeOwner::binding_compaction_authority(&owner);
    let pins = crate::physical_runtime::record_serving::SelectedBlobManifestPins::empty_for_test();
    let pending = comp
        .begin_binding_compaction(checkpoint, 1, pins, 4096)
        .unwrap()
        .release_registry();
    // Fresh mutation admitted while candidate file sync holds no registry lock.
    assert!(auth
        .admit_unallocated_with(k2, fp2, || Ok::<_, ()>(m2))
        .is_ok());
    assert_eq!(owner.binding_counts(), (2, 2));
    comp.resume_binding_compaction(pending)
        .unwrap()
        .commit_for_test()
        .unwrap();
    assert_eq!(owner.binding_counts(), (2, 1));
    f.media.close();

    // Reopened saturated registry (live == live_limit, admitted == 0) with a Terminal binding.
    let mut f2 = fixture(16);
    for i in 0u8..16 {
        let k = f2
            .registry
            .issue_key(mat([i.wrapping_add(80); 32]))
            .unwrap();
        let k_id = k.identity();
        let fp = fingerprint(&f2, i + 1);
        let m = mutation(&f2, u64::from(i) + 1);
        assert!(matches!(
            f2.registry.admit_unallocated(k, fp, m),
            Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
        ));
        if i == 0 {
            let obs = PhysicalMutationUnresolvedBindingObservation::new(k_id, fp, m);
            f2.registry
                .cancel_before_group_seal(
                    obs,
                    PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
                )
                .unwrap();
        }
    }
    f2.registry.set_namespace_durable_generation_for_test(1);
    assert_eq!(
        PhysicalMutationIdempotencyRuntimeOwner::from_rebuilt_registry(f2.registry)
            .binding_counts(),
        (16, 1)
    );
    f2.media.close();
}
