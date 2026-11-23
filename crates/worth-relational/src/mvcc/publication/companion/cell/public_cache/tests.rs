//! Opaque cache destruction must run beyond the exact owner guards.
use super::*;
use crate::identity::data::VersionId;
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
        assert!(self.core.upgrade().unwrap().current.try_lock().is_ok());
        assert!(self
            .image
            .upgrade()
            .unwrap()
            .public_caches
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
    let selected = PositionedRelationalSnapshot {
        runtime_instance_id: 1,
        branch_id: BranchId("cache-retirement".into()),
        root_id: 7,
        version_id: VersionId(1),
        commit_id: Some(CommitId(3)),
        position: Some(PatchStreamPosition(3)),
    };
    let image = Arc::new(ManagedIndexBranchImage::default());
    let cell = CompanionBranchCell::new(&selected, generation, Arc::clone(&image));
    let root = ManagedSourceRootImage {
        core: Arc::downgrade(&cell.core),
        affinity: cell.core.retain_source_affinity(&execution).unwrap(),
        image: Arc::clone(&cell.core.current.lock().unwrap()),
        _fork_memory: None,
    };
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
    let current = cell.core.current.lock().unwrap();
    let retired = cache.retire();
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop((current, epoch));
    drop(cache);
    drop(retired);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(image.public_caches.lock().unwrap().entries.is_empty());
    drop((root, cell, image, registry));
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
