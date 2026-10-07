//! The canonical subscription forwards Native head admission and contention unchanged.
use super::*;
use crate::relational_source::relational_test_support::{
    create_entity_outcome, release_test_commit_snapshot, runtime_with_declared_aspect_schema,
};
use std::sync::{mpsc, Mutex};
use worth_relational::facade::{config::CascadeDeletePolicy, runtime::RelationalRuntime};

#[derive(Debug)]
struct Consumer {
    gate: Option<(mpsc::Sender<()>, Mutex<mpsc::Receiver<()>>)>,
}
impl RelationalPublicationCompanion for Consumer {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let cell = context.mint_selected_branch_cell(Arc::new(0_u64))?;
        let reserved = cell.reserve_preflight(context)?;
        let effect = context.seal_replacement(reserved, Arc::new(1_u64))?;
        if let Some((prepared, release)) = &self.gate {
            prepared.send(()).unwrap();
            release.lock().unwrap().recv().unwrap();
        }
        Ok(effect)
    }
}
fn runtime() -> Arc<RelationalRuntime> {
    let runtime = Arc::new(runtime_with_declared_aspect_schema(
        CascadeDeletePolicy::CascadeDeleteRelations,
    ));
    let baseline = create_entity_outcome(&runtime, "canonical-head-baseline");
    release_test_commit_snapshot(&runtime, &baseline);
    runtime
}
fn subscribe(
    runtime: &Arc<RelationalRuntime>,
    consumer: Consumer,
) -> RelationalBridgeCanonicalSubscription {
    RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(runtime), "head-cell-test")
        .unwrap()
        .begin_canonical_envelope_subscription()
        .unwrap()
        .activate(
            Arc::new(consumer),
            CompanionPreflightBudget {
                maximum_work_visits: 32,
                maximum_preparation_bytes: 8192,
            },
        )
        .unwrap()
}

#[test]
fn canonical_subscription_head_cell_selects_the_current_native_head() {
    let runtime = runtime();
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let old_handle = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let old = runtime
        .read_truth()
        .positioned_snapshot(&old_handle)
        .unwrap();
    let subscription = subscribe(&runtime, Consumer { gate: None });
    let written = create_entity_outcome(&runtime, "canonical-new-head");
    let head = runtime
        .read_truth()
        .positioned_snapshot(&written.snapshot)
        .unwrap();
    assert_ne!(old.root_id(), head.root_id());
    let cell = subscription
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(2_u64),
            |cell| cell,
        )
        .unwrap();
    assert_eq!(cell.read_image().root_id(), head.root_id());
    assert_eq!(cell.read_image().position(), head.position());
    release_test_commit_snapshot(&runtime, &written);
    runtime.snapshots().release_snapshot(&old_handle).unwrap();
}

#[test]
fn canonical_subscription_head_cell_forwards_contention_without_install_or_custody() {
    let runtime = runtime();
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let old_handle = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let (prepared, observe_prepared) = mpsc::channel();
    let (release, observe_release) = mpsc::channel();
    let subscription = subscribe(
        &runtime,
        Consumer {
            gate: Some((prepared, Mutex::new(observe_release))),
        },
    );
    let initial = Arc::new(3_u64);
    let weak = Arc::downgrade(&initial);
    let mut installed = false;
    std::thread::scope(|threads| {
        let publisher = threads.spawn(|| create_entity_outcome(&runtime, "canonical-held-head"));
        observe_prepared.recv().unwrap();
        let result = subscription.with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            initial,
            |_| {
                installed = true;
            },
        );
        release.send(()).unwrap();
        let committed = publisher.join().unwrap();
        assert_eq!(
            result,
            Err(PublicationCompanionRegistrationStop::HeadCellPublicationContended)
        );
        assert!(!installed);
        assert!(weak.upgrade().is_none());
        release_test_commit_snapshot(&runtime, &committed);
    });
    runtime.snapshots().release_snapshot(&old_handle).unwrap();
}
