//! Actual selected cold owners close while original retired payloads remain alive.
use super::*;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::{
    RelationalBranchSharingInspectionDenial as Inventory,
    RelationalRetainedStorageInspectionDenial as Census,
};
use worth_relational::facade::storage::{
    RelationalStorageSubject as Subject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};

fn writer(
    path: &std::path::Path,
    execution: ExecutionRequest<'_, '_>,
) -> worth_query_host::facade::WorthQueryApplicationHomeClosed {
    let home = open(
        path,
        execution,
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    for key in 0..4 {
        assert!(matches!(
            operations::set_key(home.runtime(), 0xc105e + key, 8),
            WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
        ));
    }
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("original Host writer close: {refused:?}"));
    assert!(closed.is_closed());
    closed
}
#[test]
fn cold_and_materialized_original_sources_physically_close_while_metadata_and_retired_payloads_live(
) {
    for materialize in [false, true] {
        let path = tempfile::tempdir().unwrap();
        let live = serial(None, None);
        let execution = ExecutionRequest::serial(&live);
        let writer = writer(path.path(), execution);
        with_bootstrap_advancement_from_request(execution, |phase| {
            let (query, home) = super::super::unshared(path.path(), &phase);
            let home = home.share().consume(|home| home);
            // Native original-envelope admission may read canonical members
            // (D12). Prove cold record custody directly, without demanding it.
            let before_probe = home.relational().cold_storage_reads();
            let cold = home.relational().runtime().observe_retained_storage();
            assert!(
                matches!(
                    &cold,
                    Err(Census::OwnerInventory(Inventory::StorageUnavailable(
                        RelationalStorageUnavailable {
                            subject: Subject::Region(_),
                            cause: Cause::SourceUnavailable
                        }
                    )))
                ),
                "the genuine original region remains cold: {cold:?}"
            );
            assert_eq!(
                home.relational().cold_storage_reads(),
                before_probe,
                "the coldness probe performs no storage read"
            );
            let pins = if materialize {
                let native = home.relational().runtime();
                let (_, basis) = native
                    .observe_branch(&native.main_branch_identity())
                    .unwrap();
                let snapshot = native
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                drop(basis);
                let view = native
                    .read_truth()
                    .read_snapshot(&snapshot, &execution)
                    .unwrap()
                    .unwrap();
                let entity = view
                    .entities()
                    .first()
                    .expect("actual authored Native rows")
                    .entity_id;
                let history = native
                    .history()
                    .entity_aspect_history(&execution, &BranchId("main".into()), entity, None)
                    .unwrap();
                assert!(
                    !history.is_empty(),
                    "actual historical members were demanded"
                );
                let pin = native
                    .read_truth()
                    .pin_resident_entity_metadata(&snapshot, entity)
                    .unwrap()
                    .unwrap();
                let kind = pin.live_kind().expect("an actual resident live entity");
                let alias = pin.clone();
                native.snapshots().release_snapshot(&snapshot).unwrap();
                drop((snapshot, view, history));
                Some((pin, alias, kind))
            } else {
                None
            };
            drop(query);
            let closed = home.close(&execution);
            let allocation = match closed {
                ApplicationHomeCloseOutcome::Closed { allocation, .. } => allocation,
                ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
                    panic!("original cold owner retirement: {reason:?}")
                }
            };
            assert_physical_lease_released(path.path(), &execution);
            if let Some((pin, alias, kind)) = pins {
                assert_eq!(pin.live_kind(), Some(kind));
                allocation.reclaim();
                drop(pin);
                assert_eq!(
                    alias.live_kind(),
                    Some(kind),
                    "independent metadata survives original owner reclaim"
                );
                drop(alias);
            } else {
                allocation.reclaim();
            }
        })
        .unwrap();
        writer.reclaim();
    }
}
#[test]
fn an_original_source_capable_fork_refuses_before_native_seal_and_same_home_retry_closes() {
    let path = tempfile::tempdir().unwrap();
    let live = serial(None, None);
    let execution = ExecutionRequest::serial(&live);
    let writer = writer(path.path(), execution);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, home) = super::super::unshared(path.path(), &phase);
        let home = home.share().consume(|home| home);
        let fork = home.relational().runtime().fork().unwrap();
        drop(query);
        let original_identity = home.home_identity();
        let refused = match home.close(&execution) {
            ApplicationHomeCloseOutcome::CloseRefused {
                reason:
                    ApplicationHomeCloseDenial::Relational(
                        worth_store_relational::RelationalSectionCloseDenial::Source(
                            worth_relational::facade::runtime::RelationalSourceCloseDenial::Busy,
                        ),
                    ),
                posture: ApplicationHomeClosePosture::Serving(custody),
            } => custody,
            ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
                panic!("wrong original fork refusal: {reason:?}")
            }
            ApplicationHomeCloseOutcome::Closed { .. } => {
                panic!("a live source-capable fork cannot be declared retired")
            }
        };
        assert_eq!(refused.home_identity(), original_identity);
        crate::incarnation::assert_contended(path.path());
        drop(fork);
        let allocation = match refused.close(&execution) {
            ApplicationHomeCloseOutcome::Closed { allocation, .. } => allocation,
            ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
                panic!("same original Home after fork release: {reason:?}")
            }
        };
        assert_physical_lease_released(path.path(), &execution);
        allocation.reclaim();
    })
    .unwrap();
    writer.reclaim();
}
