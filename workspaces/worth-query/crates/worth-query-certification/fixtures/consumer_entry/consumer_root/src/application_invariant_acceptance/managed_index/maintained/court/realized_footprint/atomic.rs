//! Interfere after successful target verification, outside the Query branch lane.
use super::super::super::schema::ItemName;
use super::*;
use std::{num::NonZeroUsize, time::Duration};

#[test]
fn realized_footprint_atomic_pin_rejects_a_native_write_after_target_verification() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let original = request.query(find("A", "sh")).execute().unwrap();
        let source = original.result_set_observation().unwrap().clone();
        assert!(source.has_verified_realized_footprint());
        let (partition, slot, generation) = original.rows()[0].coordinate;
        let record = RelationalBridgeRecordIdentityParts::entity(partition, slot, generation);
        // The later target must be Realized, rather than the unchanged whole-Source path.
        insert(
            &world,
            &principal,
            &scope,
            "irrelevant",
            "zz-other",
            0x919a_7200,
        );
        let pause = world
            .runtime()
            .pause_after_managed_source_revalidation_for_test(NonZeroUsize::MIN);
        let outcome = std::thread::scope(|threads| {
            let runtime = world.runtime();
            let interfere = threads.spawn(move || {
                assert!(
                    pause.wait_until_reached(Duration::from_secs(20)),
                    "real target verifier never reached the pin seam"
                );
                let scope = authentication::request_scope();
                runtime
                    .publish_native_field_write_for_test(
                        runtime.current_world(),
                        record,
                        ItemName::reference(),
                        "shifted".to_owned(),
                        &scope,
                    )
                    .unwrap();
                pause.release();
            });
            let outcome = request
                .advanced()
                .mutate(rename("sh", "wrong"))
                .expect_result_set(source)
                .idempotency(&0x919a_7201u64)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            interfere.join().unwrap();
            outcome
        });
        let Some(WorthQueryApplicationUncommitted::Deferred(refused)) = outcome.commit_outcome()
        else {
            panic!(
                "expected Native pin refusal, got {:?}",
                outcome.commit_outcome()
            );
        };
        assert!(
            matches!(
                refused.kind(),
                WorthQueryApplicationCommitDeferredKind::RelationalDeferred(
                    RelationalPublicationDeferred::CompanionPreflight(
                        CompanionPreflightStop::SelectedSourceMismatch,
                    )
                )
            ),
            "wrong refusal: {:?}",
            refused.kind()
        );
        assert_eq!(values(original.rows()), vec![("x".into(), "shared".into())]);
        assert_eq!(
            values(request.query(find("A", "sh")).execute().unwrap().rows()),
            vec![("x".into(), "shifted".into())]
        );
    });
}
