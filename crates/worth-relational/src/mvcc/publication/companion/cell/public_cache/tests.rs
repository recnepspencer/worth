//! Opaque cache destruction must run beyond the exact owner guards.
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::positioned_snapshot::{assemble_positioned_relational_snapshot, PositionedRelationalSnapshotParts};
use super::super::*;
use crate::identity::data::VersionId;
use crate::validation::candidate_source::image::ManagedIndexBranchImage;
use std::sync::Weak;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::CompanionBranchCellCoreFixturePort as _;
use worth_relational_branch::internal_port::validation::candidate_source::image::ManagedIndexBranchImageFieldsPort as _;

type Payload = Arc<dyn Any + Send + Sync>;
use crate::mvcc::publication::companion::registration::CompanionRegistry;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

struct Probe {
    core: Weak<CompanionBranchCellCore>,
    image: Weak<ManagedIndexBranchImage>,
    registry: Arc<CompanionRegistry>,
    drops: Arc<AtomicUsize>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        assert!(self
            .core
            .upgrade()
            .unwrap()
            .try_lock_unpoisoned_image_for_test()
            .is_some());
        assert!((*self.image.upgrade().unwrap().owner_public_caches())
            .try_lock()
            .is_ok());
        self.registry
            .begin_required()
            .expect("last opaque Drop occurs beyond the genuine Native registration epoch");
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn public_cache_retirement_defers_opaque_drop_and_releases_original_backing() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let registry = Arc::new(CompanionRegistry::new());
    let generation = registry.begin_required().unwrap();
    let selected = assemble_positioned_relational_snapshot(PositionedRelationalSnapshotParts {
        runtime_instance_id: 1,
        branch_id: BranchId("cache-retirement".into()),
        root_id: 7,
        version_id: VersionId(1),
        commit_id: Some(CommitId(3)),
        position: Some(PatchStreamPosition(3)),
    });
    let image = Arc::new(ManagedIndexBranchImage::default());
    let cell = CompanionBranchCell::new(&selected, generation, Arc::clone(&image));
    cell.core.retain_source_affinity(&execution).unwrap();
    let root = ManagedSourceRootImage::capture_current(&cell.core).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let payload: Payload = Arc::new(Probe {
        core: Arc::downgrade(&cell.core),
        image: Arc::downgrade(&image),
        registry: Arc::clone(&registry),
        drops: Arc::clone(&drops),
    });
    let cache = root
        .retain_public_cache(&execution, selected, generation, &payload, None)
        .unwrap();
    drop(payload);
    let epoch = registry.enter().unwrap();
    let current = cell.core.lock_unpoisoned_image_for_test();
    let retired = cache.retire();
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop((current, epoch));
    drop(cache);
    drop(retired);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!((*image.owner_public_caches())
        .lock()
        .unwrap()
        .entries
        .is_empty());
    drop((root, cell, image, registry));
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
