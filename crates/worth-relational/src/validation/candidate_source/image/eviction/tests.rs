//! Destruction of the final opaque value must not run under its directory lock.
use super::*;
use crate::tests::support::{create_entity, managed_cache_schema, snapshot_for_owner_identity};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Weak,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
struct Probe {
    image: Arc<Mutex<Weak<ManagedIndexBranchImage>>>,
    drops: Arc<AtomicUsize>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        let image = self.image.lock().unwrap().upgrade().unwrap();
        assert!(
            image.roots.try_lock().is_ok(),
            "opaque Drop is outside directory mutex"
        );
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn managed_cache_retirement_defers_last_opaque_drop_beyond_directory_guard() {
    let (schema, program) = managed_cache_schema();
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let scope = create_entity(&execution, &runtime, "scope");
    let weak = Arc::new(Mutex::new(Weak::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    let payload: Payload = Arc::new(Probe {
        image: weak.clone(),
        drops: drops.clone(),
    });
    let snapshot = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let revision = runtime
        .read_truth()
        .positioned_snapshot(&snapshot)
        .unwrap()
        .commit_id()
        .unwrap();
    let mut image = ManagedIndexBranchImage::default();
    image
        .prepare_source(
            &execution,
            &program,
            scope,
            Availability::Available,
            revision,
            true,
            Some(&payload),
        )
        .unwrap();
    let image = ManagedIndexBranchImage::retain_header(&execution, image).unwrap();
    *weak.lock().unwrap() = Arc::downgrade(&image);
    drop(payload);
    let (_, foreign) = managed_cache_schema();
    assert!(matches!(
        image.retained(&foreign, scope),
        Err(
            crate::validation::candidate_source::ManagedIndexSourcePreparationDenial::ForeignSource
        )
    ));
    assert!(matches!(
        image.evict_source(&execution, &foreign, scope),
        Err(Stop::ForeignCell)
    ));
    let retired = image.evict_source(&execution, &program, scope).unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert!(image
        .retained(&program, scope)
        .unwrap()
        .unwrap()
        .payload()
        .is_none());
    drop(retired);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
