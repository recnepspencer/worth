#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::readiness::*;
use tempfile::tempdir;
use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::PhysicalWorkPreEffectDenial,
    worth_store::physical_runtime::PhysicalWorkReadiness,
    worth_store::physical_runtime::PhysicalWorkSubmissionOutcome,
    worth_store::physical_runtime::PhysicalWorkSubmissionReceipt,
};

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::family_locality_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::matching_aspect_delta,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::EXPECTED_NATIVE_RECORD_BINDING_COUNT,
};

#[test]
fn store_admission_precedes_signal_lineage_and_both_are_effect_free() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let receipt = success(serving.physical_read_submission().submit(read_request));
    let identity = receipt.identity();
    let before = serving.media_counters();

    let admitted = serving.admit_physical_work(receipt).unwrap();
    assert_eq!(admitted.intent().identity(), receipt.identity());
    let ready = match serving.request_physical_work(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "declared read dependencies unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    };
    assert_eq!(ready.intent().identity(), receipt.identity());
    assert!(!ready.capability_registry_digest().is_empty());
    assert!(!ready.capability_bundle_digest().is_empty());
    assert!(!ready.payload_contract_digest().is_empty());
    assert_eq!(serving.media_counters(), before);

    assert!(matches!(
        serving.revalidate_physical_work(ready).unwrap(),
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
    ));
    assert_eq!(serving.media_counters(), before);
    assert_eq!(
        serving.admit_physical_work(receipt).err(),
        Some(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::CommandAbsent)
    );
    await_no_active_signal(&serving);
    let closed = serving.close();
    assert_eq!(closed.work().declared(), 1);
    assert_eq!(closed.work().residual(), 0);
    assert_eq!(
        closed.work().drain().released_before_dispatch(),
        &[identity]
    );
    assert_eq!(closed.work().unaccounted_terminal(), 0);
}

#[test]
fn stale_generation_and_unhealthy_serving_deny_before_signal_or_media() {
    let stale_root = tempdir().unwrap();
    let unhealthy_root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let stale = serving_from_initialization_with_work_profile(stale_root.path(), profile.clone());
    let unhealthy = serving_from_initialization_with_work_profile(unhealthy_root.path(), profile);
    let stale_receipt = success(
        stale
            .physical_read_submission()
            .submit(read_request.clone()),
    );
    let unhealthy_receipt = success(unhealthy.physical_read_submission().submit(read_request));
    let stale_media = stale.media_counters();
    let unhealthy_media = unhealthy.media_counters();

    stale.certification_begin_lifecycle_termination();
    unhealthy.certification_require_serving_inspection();
    assert_eq!(
        stale.admit_physical_work(stale_receipt).err(),
        Some(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::StaleGeneration)
    );
    assert_eq!(
        unhealthy.admit_physical_work(unhealthy_receipt).err(),
        Some(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::UnhealthyServing)
    );
    assert_eq!(stale.media_counters(), stale_media);
    assert_eq!(unhealthy.media_counters(), unhealthy_media);
    stale.close();
    unhealthy.close();
}

#[test]
fn a_foreign_store_receipt_cannot_enter_signal() {
    let first_root = tempdir().unwrap();
    let second_root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let first = serving_from_initialization_with_work_profile(first_root.path(), profile.clone());
    let second = serving_from_initialization_with_work_profile(second_root.path(), profile);
    let receipt = success(first.physical_read_submission().submit(read_request));
    let before = second.media_counters();

    assert_eq!(
        second.admit_physical_work(receipt).err(),
        Some(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ForeignStore)
    );
    assert_eq!(second.media_counters(), before);
    first.close();
    second.close();
}

#[test]
fn admitted_work_is_fenced_again_before_signal_request() {
    let first_root = tempdir().unwrap();
    let second_root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let first = serving_from_initialization_with_work_profile(first_root.path(), profile.clone());
    let second = serving_from_initialization_with_work_profile(second_root.path(), profile);
    let receipt = success(first.physical_read_submission().submit(read_request));
    let admitted = first.admit_physical_work(receipt).unwrap();
    let before = second.media_counters();

    assert!(matches!(
        second.request_physical_work(admitted),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ForeignStore)
    ));
    assert_eq!(second.media_counters(), before);
    first.close();
    second.close();
}

#[test]
fn foreign_signal_handle_collisions_cannot_revalidate_store_work() {
    let first_root = tempdir().unwrap();
    let second_root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let first = serving_from_initialization_with_work_profile(first_root.path(), profile.clone());
    let second = serving_from_initialization_with_work_profile(second_root.path(), profile);
    let first_ready = ready_read(&first, read_request.clone());
    let second_ready = ready_read(&second, read_request);
    assert_eq!(
        first_ready.signal_request(),
        second_ready.signal_request(),
        "the hostile case must exercise equal runtime-local Signal handles"
    );
    let before = second.media_counters();

    assert!(matches!(
        second.revalidate_physical_work(first_ready),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ForeignStore)
    ));
    assert!(matches!(
        second.revalidate_physical_work(second_ready).unwrap(),
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
    ));
    assert_eq!(second.media_counters(), before);
    first.close();
    second.close();
}

#[test]
fn lifecycle_change_between_admission_and_request_revokes_the_work() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let receipt = success(serving.physical_read_submission().submit(read_request));
    let admitted = serving.admit_physical_work(receipt).unwrap();
    let before = serving.media_counters();

    serving.certification_begin_lifecycle_termination();
    assert!(matches!(
        serving.request_physical_work(admitted),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::StaleGeneration)
    ));
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn changed_declared_dependency_is_evaluated_before_readiness_without_media() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    serving
        .certification_apply_physical_aspect_delta(matching_aspect_delta(1, "dirty"))
        .unwrap();
    let admitted = serving
        .admit_physical_work(success(
            serving.physical_read_submission().submit(read_request),
        ))
        .unwrap();
    let before = serving.media_counters();

    let readiness = serving.request_physical_work(admitted).unwrap();
    assert!(matches!(
        readiness,
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
    ));
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn dependency_invalidation_evaluates_only_its_locality_owner() {
    let root = tempdir().unwrap();
    let (profile, read_request, write_request, read_delta) = family_locality_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    serving
        .certification_apply_physical_aspect_delta(read_delta)
        .unwrap();
    let read = serving
        .admit_physical_work(success(
            serving.physical_read_submission().submit(read_request),
        ))
        .unwrap();
    let write_receipt = match serving
        .physical_mutation_submission()
        .submit(write_request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should declare: {outcome:?}"),
    };
    let write = serving.admit_physical_work(write_receipt).unwrap();
    let before = serving.media_counters();

    assert!(matches!(
        serving.request_physical_work(read).unwrap(),
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
    ));
    assert!(matches!(
        serving.request_physical_work(write).unwrap(),
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
    ));
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn changed_dependency_revalidation_refreshes_the_exact_active_lineage_without_media() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let ready = ready_read(&serving, read_request);
    let active = ready.signal_request();
    serving
        .certification_apply_physical_aspect_delta(matching_aspect_delta(
            1,
            "changed-after-readiness",
        ))
        .unwrap();
    let before = serving.media_counters();

    let refreshed = match serving.revalidate_physical_work(ready).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => {
            panic!("evaluated dependency must become ready")
        }
    };
    assert_eq!(refreshed.intent().identity().operation().get(), 1);
    assert_ne!(
        refreshed.signal_request(),
        active,
        "changed dependency must supersede active readiness"
    );
    assert_eq!(
        refreshed.revalidated_from_signal_request(),
        Some(active),
        "Store must carry Signal's exact supersession lineage"
    );
    let supersession = refreshed
        .supersession()
        .expect("revalidation must retain Signal's supersession proof");
    assert_eq!(supersession.signal().previous(), active);
    assert_eq!(
        supersession.signal().replacing(),
        refreshed.signal_request()
    );
    assert_eq!(
        supersession.previous_obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched
    );
    assert_eq!(
        refreshed.signal_request().generation().get(),
        active.generation().get() + 1
    );
    assert_eq!(
        refreshed.signal_request().branch_epoch(),
        active.branch_epoch()
    );
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn rebuilding_the_signal_owner_preserves_readiness_descriptors_without_media_effects() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let first = serving_from_initialization_with_work_profile(root.path(), profile.clone());
    let first_before = first.media_counters();
    let first_ready = ready_read(&first, read_request.clone());
    let first_descriptors = readiness_descriptors(&first_ready);
    assert_eq!(first.media_counters(), first_before);
    first.close();

    let rebuilt = serving_from_open_with_work_profile(root.path(), profile);
    let rebuilt_before = rebuilt.media_counters();
    let rebuilt_ready = ready_read(&rebuilt, read_request);
    assert_eq!(readiness_descriptors(&rebuilt_ready), first_descriptors);
    assert_eq!(rebuilt.media_counters(), rebuilt_before);
    rebuilt.close();
}

#[test]
fn signal_observation_is_bounded_and_contains_no_runtime_handles() {
    let root = tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    let declared_profile = profile.identity();
    let declared_binding_count = u16::try_from(profile.contract_count()).unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile.clone());
    let observation = serving.physical_signal_observation().unwrap();
    assert_ne!(
        observation.profile(),
        declared_profile,
        "runtime observation must identify the installed profile, including native bindings"
    );
    assert_eq!(observation.graph_owner_count(), 1);
    let installed_binding_count = declared_binding_count + EXPECTED_NATIVE_RECORD_BINDING_COUNT;
    assert_eq!(observation.aspect_binding_count(), installed_binding_count);
    assert_eq!(observation.locality_owner_count(), installed_binding_count);
    assert_eq!(
        observation.async_family_count(),
        9,
        "the bounded async capability table includes checkpoint capture and WAL reclamation among nine exact families"
    );
    assert_eq!(observation.clock().current_tick(), 0);
    serving.close();

    let rebuilt = serving_from_open_with_work_profile(root.path(), profile);
    let rebuilt_observation = rebuilt.physical_signal_observation().unwrap();
    assert_eq!(
        rebuilt_observation.profile(),
        observation.profile(),
        "the installed profile identity must be stable across runtime reconstruction"
    );
    assert_eq!(
        rebuilt_observation.aspect_binding_count(),
        installed_binding_count
    );
    rebuilt.close();
}
