//! Genuine pending release effect backing survives Core and seal transfer,
//! then Serving consumes it through the same native Recovery pool.

use super::*;

#[test]
fn genuine_pending_effect_backing_moves_through_serving_and_checkpoint() {
    std::thread::Builder::new()
        .name("pending-effect-pool-handoff".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = pending_wal_world::first();
            world.kill_distinct_release_before_checkpoint();
            let root = world.root();
            let outcome = WorthStoreRecovery::recover(super::super::super::request(root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("second genuine pending V3 must recover: {outcome:?}")
            };
            assert!(
                handoff.core().recovery_effect_count() > 0,
                "the killed second release requires actual C8 redo"
            );
            let core = handoff.into_core();
            let policy = core.residency_policy();
            let format = AdmittedPhysicalRecordFormat::admit(policy.record_format());
            let allocations = core.certification_residency_allocations();
            let effect_bytes = core
                .selected_head_effect_owned_heap_bytes()
                .expect("actual V14 effect witness capacity");
            assert!(
                effect_bytes > 0,
                "pending V3 effects retain native witnesses"
            );
            let fingerprint_bytes = backing_census::retained_fingerprint_bytes(&core);
            assert!(fingerprint_bytes >= effect_bytes);
            let checkpoint_bytes = core.checkpoint().map_or(0, |checkpoint| {
                checkpoint
                    .owned_heap_bytes()
                    .expect("actual selected checkpoint backing")
            });
            let consumed_bytes = fingerprint_bytes + checkpoint_bytes;

            let before_seal = allocations.snapshot();
            let seal = core
                .into_checkpoint_custody()
                .expect("real pending C8 claim has an independent Store seal");
            let after_seal = allocations.snapshot();
            assert_eq!(after_seal.pool(), before_seal.pool());
            let scope = Dimension::OperationScope(Scope::Recovery);
            assert_eq!(
                after_seal.for_dimension(scope).active_units(),
                before_seal.for_dimension(scope).active_units(),
                "native effect and head witnesses remain owned through the seal"
            );

            let TransitionOutcome::Success(serving) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("real pending seal must open Serving")
            };
            let after_serving = allocations.snapshot();
            assert_eq!(after_serving.pool(), before_seal.pool());
            assert_eq!(
                after_serving.for_dimension(scope).active_units(),
                after_seal.for_dimension(scope).active_units() - consumed_bytes,
                "Serving consumes its checkpoint and WAL, head, and effect witnesses"
            );
            assert_eq!(
                after_serving.for_dimension(scope).released_units()
                    - after_seal.for_dimension(scope).released_units(),
                consumed_bytes + after_serving.for_dimension(scope).admitted_units()
                    - after_seal.for_dimension(scope).admitted_units(),
                "temporary Serving reads balance in the same native pool"
            );
            checkpoint(&serving, [0xe4; 32]);
            serving.close();
            backing_census::assert_disposed("pending effect Serving", |dimension| {
                let counters = allocations.snapshot().for_dimension(dimension);
                (
                    counters.active_units(),
                    counters.admitted_units(),
                    counters.released_units(),
                )
            });

            let fresh = recover_core(root);
            let seal = fresh
                .into_checkpoint_custody()
                .expect("checkpointed release custody reopens");
            let TransitionOutcome::Success(serving) =
                open_with_policy(root, seal, format, policy).into_raw()
            else {
                panic!("fresh checkpointed custody must open Serving")
            };
            serving.close();
        })
        .unwrap()
        .join()
        .expect("pending effect handoff worker");
}
