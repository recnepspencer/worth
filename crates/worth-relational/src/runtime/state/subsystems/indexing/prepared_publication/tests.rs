//! Catalog/coordination kernels use ordering inputs, not forged commit truth.
use super::*;
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexEntries, DerivedIndexGenerationId, DerivedIndexId,
    DerivedIndexPublicationStatus,
};
use crate::schema::data::SchemaVersionId;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
enum AdmissionStop {
    Cancelled,
    PermitWasNotReleased,
}

fn generation(id: u64) -> DerivedIndexGeneration {
    DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(id),
        index_id: DerivedIndexId(1),
        source_commit_id: CommitId(id + 100),
        source_branch_id: BranchId("main".into()),
        applicability: DerivedIndexApplicability {
            branch_id: BranchId("main".into()),
            version_id: VersionId(id),
            schema_version: SchemaVersionId(1),
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(BTreeMap::new().into()),
    }
}

#[test]
fn deferred_admission_prices_the_catalog_after_intervening_publication() {
    let empty = IndexingSubsystem::default();
    let mut small = empty
        .prepare_generation_batch(vec![generation(200)], None, |_, _| Ok::<_, ()>(()))
        .unwrap();
    let mut small_work = 0;
    small
        .admit_actual_geometry(|work, _| {
            small_work += work;
            Ok::<_, ()>(())
        })
        .unwrap();
    drop(small);
    let grown = IndexingSubsystem::default();
    let mut batch = grown
        .prepare_generation_batch(vec![generation(200)], None, |_, _| Ok::<_, ()>(()))
        .unwrap();
    for id in 1..128 {
        grown.publish_generation(generation(id));
    }
    let mut actual_work = 0;
    batch
        .admit_actual_geometry(|work, _| {
            actual_work += work;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert!(
        actual_work > small_work,
        "admission prices actual retained geometry, not usize::MAX"
    );
    assert!(
        grown
            .state
            .read()
            .generations
            .generation(DerivedIndexGenerationId(200))
            .is_none(),
        "admission has not published the prepared generation"
    );
}

#[test]
fn cancelled_catalog_wait_keeps_the_unpublished_payload_and_releases_no_foreign_permit() {
    let owner = IndexingSubsystem::default();
    let mut first = owner
        .prepare_generation_batch(vec![generation(1)], None, |_, _| Ok::<_, ()>(()))
        .unwrap();
    first.admit_actual_geometry(|_, _| Ok::<_, ()>(())).unwrap();
    let mut second = owner
        .prepare_generation_batch(vec![generation(2)], None, |_, _| Ok::<_, ()>(()))
        .unwrap();
    let mut polls = 0;
    let denied = second.admit_actual_geometry(|_, _| {
        polls += 1;
        if polls == 1 {
            Ok(())
        } else {
            Err(AdmissionStop::Cancelled)
        }
    });
    assert_eq!(polls, 2, "a contended admission rechecks original controls");
    assert!(matches!(
        denied,
        Err(SelectedIndexGenerationAdmissionStop::Admission(
            AdmissionStop::Cancelled
        ))
    ));
    assert_eq!(second.generations.len(), 1);
    assert!(!second.is_published());
    assert!(owner
        .state
        .read()
        .generations
        .generation(DerivedIndexGenerationId(2))
        .is_none());
    let mut first_poll = true;
    let still_held = owner.generation_publication.enter(|| {
        if std::mem::take(&mut first_poll) {
            Ok(())
        } else {
            Err(AdmissionStop::Cancelled)
        }
    });
    assert!(
        matches!(still_held, Err(AdmissionStop::Cancelled)),
        "a refused waiter did not release the first owner's permit"
    );
    drop(first);
    let mut first_poll = true;
    let released = owner
        .generation_publication
        .enter(|| {
            if std::mem::take(&mut first_poll) {
                Ok(())
            } else {
                Err(AdmissionStop::PermitWasNotReleased)
            }
        })
        .expect("dropping admission frees its gate before another owner waits");
    drop(released);
    second
        .admit_actual_geometry(|_, _| Ok::<_, AdmissionStop>(()))
        .unwrap();
}

#[test]
fn an_empty_admitted_batch_completes_without_installing_a_generation() {
    let owner = IndexingSubsystem::default();
    let mut batch = owner
        .prepare_generation_batch(Vec::new(), None, |_, _| Ok::<_, ()>(()))
        .unwrap();
    batch.admit_actual_geometry(|_, _| Ok::<_, ()>(())).unwrap();
    batch.publish_admitted();
    assert!(batch.is_published());
    assert!(owner
        .state
        .read()
        .generations
        .generation(DerivedIndexGenerationId(1))
        .is_none());
}

#[test]
fn admitted_installation_retains_payload_backing_and_unused_keys_until_retirement() {
    let owner = IndexingSubsystem::default();
    owner.publish_generation(generation(1));
    let mut batch = owner
        .prepare_generation_batch(vec![generation(2)], None, |_, _| Ok::<_, ()>(()))
        .unwrap();
    let capacity = batch.generations.capacity();
    batch.admit_actual_geometry(|_, _| Ok::<_, ()>(())).unwrap();
    batch.publish_admitted();
    assert!(batch.is_published());
    assert!(batch.generations.is_empty());
    assert_eq!(batch.generations.capacity(), capacity);
    assert!(
        batch.branch_keys[0].is_some(),
        "an unused owned lookup key stays parked for cleanup"
    );
    assert!(owner
        .state
        .read()
        .generations
        .generation(DerivedIndexGenerationId(2))
        .is_some());
    let _next = owner
        .generation_publication
        .enter(|| Ok::<_, ()>(()))
        .unwrap();
}
