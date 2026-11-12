//! No-load Head metadata pins require real region and per-runtime symbol custody.
use super::*;
use worth_relational::facade::runtime::RelationalEntityMetadataPinDenial as Denial;
#[test]
fn metadata_pin_requires_resident_head_and_never_opens_history() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let snapshot = snapshot_for_owner_branch(
        &fixture.runtime,
        &worth_relational::facade::history::BranchId("main".into()),
    );
    assert!(
        matches!(fixture.runtime.read_truth().pin_resident_entity_metadata(&snapshot, fixture.entity), Err(Denial::RegionNotResident(partition)) if partition == fixture.entity.partition_id)
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
    fixture.source.failures.lock().unwrap().extend(
        fixture
            .history
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::MissingMember)),
    );
    CurrentDoor::Entity
        .run(
            execution,
            &fixture.runtime,
            fixture.entity,
            fixture.relation,
        )
        .unwrap();
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    let metadata = fixture
        .runtime
        .read_truth()
        .pin_resident_entity_metadata(&snapshot, fixture.entity)
        .unwrap()
        .unwrap();
    assert_eq!(metadata.live_kind(), Some(KindId(1)));
    assert!(metadata
        .aspect_version(&aspect_key("name"))
        .unwrap()
        .is_some());
    assert_eq!(
        metadata.aspect_version(&aspect_key("never-installed")),
        Some(None)
    );
    assert_eq!(metadata.retirement(), None);
    fixture
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
    assert_eq!(
        metadata.live_kind(),
        Some(KindId(1)),
        "the counted pin outlives its tracked handle"
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .any(|member| fixture.history.contains(member)));
    let original_revision = metadata.aspect_version(&aspect_key("name"));
    update_entity(
        execution,
        &fixture.runtime,
        fixture.entity,
        "metadata-pin-after",
    );
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    assert_eq!(
        metadata.aspect_version(&aspect_key("name")),
        original_revision,
        "a counted Head metadata pin retains its old meaning across a commit"
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .any(|member| fixture.history.contains(member)));
}
#[test]
fn an_admitted_head_read_never_becomes_absence_after_transport_damage() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let view = fixture
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        aspect_key("name"),
        worth_foundational::facade::CanonicalFieldPath::single(field_key("name")),
    );
    let mut invalidated = false;
    let revision = view
        .entity_field_revision_admitted(execution, fixture.entity, &locator, |_| {
            if !invalidated {
                invalidated = true;
                let target = fixture.source.member_reads.lock().unwrap()[0];
                assert!(fixture.heads.contains(&target));
                fixture
                    .source
                    .failures
                    .lock()
                    .unwrap()
                    .insert(ColdRead::Member(target), Cause::Damaged);
                let failed = fixture
                    .runtime
                    .durability_authority()
                    .native_checkpoint(execution, execution.allocation_policy())
                    .unwrap_err();
                let worth_relational::facade::durability::RelationalNativeCheckpointCaptureDenial::Durability(
                    error,
                ) = failed
                else {
                    panic!("the actual export opens its selected member")
                };
                assert_eq!(error.storage_unavailable.unwrap().cause, Cause::Damaged);
                fixture.source.failures.lock().unwrap().clear();
            }
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap()
        .unwrap();
    assert!(
        revision.is_some(),
        "accepted counted Head custody cannot become an absent field"
    );
    assert!(
        matches!(view.entity_field_revision(execution, fixture.entity, &locator), Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::StorageUnavailable(failure)) if failure.cause == Cause::Damaged),
        "a fresh operation still refuses sticky damage"
    );
}

#[test]
fn fork_metadata_pin_after_public_head_read_preserves_original_aspect_revision() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let fork = fixture.runtime.fork().unwrap();
    CurrentDoor::Entity
        .run(
            execution,
            &fixture.runtime,
            fixture.entity,
            fixture.relation,
        )
        .unwrap();
    let snapshot = snapshot_for_owner_branch(
        &fork,
        &worth_relational::facade::history::BranchId("main".into()),
    );
    let original = snapshot_for_owner_branch(
        &fixture.runtime,
        &worth_relational::facade::history::BranchId("main".into()),
    );
    let expected = fixture
        .runtime
        .read_truth()
        .read_exact_snapshot_entity_aspect_version(
            execution,
            &original,
            fixture.entity,
            &aspect_key("name"),
        )
        .unwrap();
    fixture
        .runtime
        .snapshots()
        .release_snapshot(&original)
        .unwrap();
    assert_eq!(
        fork.read_truth()
            .read_exact_snapshot_entity_aspect_version(
                execution,
                &snapshot,
                fixture.entity,
                &aspect_key("name")
            )
            .unwrap(),
        expected
    );
    let metadata = fork
        .read_truth()
        .pin_resident_entity_metadata(&snapshot, fixture.entity)
        .unwrap()
        .unwrap();
    assert_eq!(metadata.live_kind(), Some(KindId(1)));
    assert_eq!(metadata.aspect_version(&aspect_key("name")), expected);
    fork.snapshots().release_snapshot(&snapshot).unwrap();
}
