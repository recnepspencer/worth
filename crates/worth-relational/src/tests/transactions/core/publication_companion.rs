use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::mvcc::{
    CompanionBranchCell, CompanionBranchCellSlot, CompanionDerivedImageRetention,
    CompanionDerivedRootAdmission, CompanionDerivedRootCost, CompanionPreflightBudget,
    CompanionPreflightStop, PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion, RelationalPublicationDeferred, RelationalPublicationOutcome,
};
use crate::tests::support::*;

mod completion_observer;
mod concurrent_head;
mod head_cell;
mod positioned_admission;
mod preflight_overflow;

struct DerivedMeter(Vec<CompanionDerivedRootCost>);

impl CompanionDerivedRootAdmission for DerivedMeter {
    type Stop = ();

    fn admit_derived_root(&mut self, cost: CompanionDerivedRootCost) -> Result<(), Self::Stop> {
        self.0.push(cost);
        Ok(())
    }
}

struct DropCounter(Arc<AtomicUsize>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct CountingCompanion {
    cell: CompanionBranchCell<u64>,
    calls: Arc<AtomicUsize>,
}

impl fmt::Debug for CountingCompanion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("CountingCompanion").finish()
    }
}

impl RelationalPublicationCompanion for CountingCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        context.claim_work(1)?;
        let reserved = self.cell.reserve_preflight(context)?;
        let next_value = *reserved.current().payload().as_ref() + 1;
        let effect = context.seal_replacement(reserved, Arc::new(next_value))?;
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(effect)
    }
}

struct FirstWriteCompanion {
    cell: Mutex<Option<CompanionBranchCellSlot<u64>>>,
    selected_position: Mutex<Option<Option<PatchStreamPosition>>>,
}

impl fmt::Debug for FirstWriteCompanion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("FirstWriteCompanion").finish()
    }
}

impl RelationalPublicationCompanion for FirstWriteCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let mut cell = self
            .cell
            .try_lock()
            .map_err(|_| CompanionPreflightStop::TopologyPending)?;
        let reserved = match cell.as_ref().and_then(CompanionBranchCellSlot::admitted) {
            Some(admitted) => admitted.reserve_preflight(context)?,
            None => {
                *self.selected_position.lock().unwrap() = Some(context.expected_position());
                let prepared = context.mint_selected_branch_cell(Arc::new(0_u64))?;
                let slot = prepared.lookup_slot();
                assert!(
                    slot.admitted().is_none(),
                    "candidate expectation is not head proof"
                );
                let reserved = prepared.reserve_preflight(context)?;
                *cell = Some(slot);
                reserved
            }
        };
        drop(cell);
        context.seal_replacement(reserved, Arc::new(1_u64))
    }
}

#[test]
fn first_write_companion_mints_from_selected_source_without_runtime_reentry() {
    let runtime = runtime_with_test_schema();
    let baseline = create_entity_outcome(&runtime, "first-write-baseline");
    let baseline_position = baseline.patch_position();
    release_test_commit_snapshot(&runtime, &baseline);

    let pending = runtime
        .publication_companion_port()
        .begin_required_registration()
        .expect("required registration starts before the first subscribed write");
    let participant = Arc::new(FirstWriteCompanion {
        cell: Mutex::new(None),
        selected_position: Mutex::new(None),
    });
    let consumer: Arc<dyn RelationalPublicationCompanion> = participant.clone();
    let _registration = pending
        .activate(
            consumer,
            CompanionPreflightBudget {
                maximum_work_visits: 16,
                maximum_preparation_bytes: 4_096,
            },
        )
        .expect("participant is active without a caller-selected branch cell");
    let written = create_entity_outcome(&runtime, "first-write-after-subscription");
    assert_eq!(
        *participant.selected_position.lock().unwrap(),
        Some(Some(baseline_position))
    );
    let cell = participant.cell.lock().unwrap();
    let image = cell
        .as_ref()
        .and_then(CompanionBranchCellSlot::admitted)
        .expect("first write installed its branch cell")
        .read_image();
    assert_eq!(**image.payload(), 1);
    assert_eq!(image.position(), Some(written.patch_position()));
    assert_eq!(image.commit_id(), Some(written.commit.commit_id));
    release_test_commit_snapshot(&runtime, &written);
}

#[test]
fn direct_publication_invokes_required_companion_and_positions_its_root() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "companion-anchor");
    let selected_handle = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let selected = runtime
        .read_truth()
        .positioned_snapshot(&selected_handle)
        .expect("owner snapshot has its canonical position");
    let original_position = selected.position();

    let registration_port = runtime.publication_companion_port();
    let pending = registration_port
        .begin_required_registration()
        .expect("registration orders behind publications");
    let cell = pending
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(7_u64),
            |cell| cell,
        )
        .expect("initial cell belongs to the selected source");
    let calls = Arc::new(AtomicUsize::new(0));
    let registration = pending
        .activate(
            Arc::new(CountingCompanion {
                cell: cell.clone(),
                calls: Arc::clone(&calls),
            }),
            CompanionPreflightBudget {
                maximum_work_visits: 16,
                maximum_preparation_bytes: 4_096,
            },
        )
        .expect("required participant is active");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("companion-direct"))
        .expect("candidate stages");
    let candidate = runtime
        .prepare_branch_transaction(transaction)
        .expect("candidate prepares");
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("direct publication must perform with its required companion");
    };
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let image = cell.read_image();
    assert_eq!(**image.payload(), 8);
    assert_eq!(
        image.commit_id(),
        Some(performed.canonical_commit().commit.commit_id)
    );
    assert_eq!(image.position(), Some(performed.patch_position()));
    assert_eq!(
        runtime
            .read_truth()
            .positioned_snapshot(&selected_handle)
            .expect("old snapshot remains positioned")
            .position(),
        original_position,
    );
    let committed = runtime
        .settle_performed_publication(performed)
        .expect("direct publication settles");
    release_test_commit_snapshot(&runtime, &committed);

    for ordinal in 0..3 {
        let later = create_entity_outcome(&runtime, &format!("unrelated-later-{ordinal}"));
        release_test_commit_snapshot(&runtime, &later);
    }
    assert_eq!(calls.load(Ordering::Relaxed), 4);
    assert_eq!(
        runtime
            .read_truth()
            .positioned_snapshot(&selected_handle)
            .expect("later commits do not displace the selected old position")
            .position(),
        original_position,
    );

    drop(registration);
    let mut later = test_owner_begin_transaction_for_main(&runtime);
    later
        .push_batch(batch_create("companion-after-client-drop"))
        .expect("later candidate stages");
    let later = runtime
        .prepare_branch_transaction(later)
        .expect("later candidate prepares");
    assert!(matches!(
        runtime.publication_port().compare_and_publish(later),
        RelationalPublicationOutcome::Deferred(
            RelationalPublicationDeferred::CompanionRebindRequired
        )
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 4);
}

#[test]
fn prepared_derived_image_keeps_conflict_and_retired_drop_custody() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "derived-image-anchor");
    let selected_handle = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let pending = runtime
        .publication_companion_port()
        .begin_required_registration()
        .expect("required registration begins");
    let cell = pending
        .with_branch_cell_at_head(
            &runtime,
            &runtime.main_branch_identity(),
            Arc::new(7_u64),
            |cell| cell,
        )
        .expect("the owner mints a real branch cell");
    let _registration = pending
        .activate(
            Arc::new(CountingCompanion {
                cell: cell.clone(),
                calls: Arc::new(AtomicUsize::new(0)),
            }),
            CompanionPreflightBudget {
                maximum_work_visits: 16,
                maximum_preparation_bytes: 4_096,
            },
        )
        .expect("the cell stays registered");
    let dropped = Arc::new(AtomicUsize::new(0));
    let mut meter = DerivedMeter(Vec::new());
    let first = cell
        .prepare_derived_root_at_same_position(
            cell.read_image(),
            Arc::new(8),
            CompanionDerivedImageRetention::from_prepared_owner(Arc::new(DropCounter(Arc::clone(
                &dropped,
            )))),
            &mut meter,
        )
        .unwrap_or_else(|_| panic!("fresh registered image prepares"));
    assert_eq!(meter.0.len(), 1);
    assert!(meter.0[0].allocation_bytes > 0);
    let (first_image, old_cleanup) = first
        .install()
        .unwrap_or_else(|_| panic!("first image installs"))
        .into_parts();
    drop(old_cleanup);
    let second = cell
        .prepare_derived_root_at_same_position(
            cell.read_image(),
            Arc::new(9),
            CompanionDerivedImageRetention::from_prepared_owner(Arc::new(())),
            &mut meter,
        )
        .unwrap_or_else(|_| panic!("second image prepares"));
    let (second_image, retired_cleanup) = second
        .install()
        .unwrap_or_else(|_| panic!("second image installs"))
        .into_parts();
    drop(first_image);
    assert_eq!(dropped.load(Ordering::Relaxed), 0);
    drop(retired_cleanup);
    assert_eq!(dropped.load(Ordering::Relaxed), 1);

    let stale_dropped = Arc::new(AtomicUsize::new(0));
    let stale = cell
        .prepare_derived_root_at_same_position(
            cell.read_image(),
            Arc::new(10),
            CompanionDerivedImageRetention::from_prepared_owner(Arc::new(DropCounter(Arc::clone(
                &stale_dropped,
            )))),
            &mut meter,
        )
        .unwrap_or_else(|_| panic!("preparing does not publish"));
    cell.replace_derived_root_at_same_position(cell.read_image(), Arc::new(11))
        .expect("a newer derived edit moves the same registered cell");
    let stopped = match stale.install() {
        Ok(_) => panic!("a stale prepared image cannot replace a newer edit"),
        Err(stopped) => stopped,
    };
    assert_eq!(
        stopped.reason(),
        crate::mvcc::CompanionCellEditStop::TopologyGenerationChanged
    );
    assert_eq!(stale_dropped.load(Ordering::Relaxed), 0);
    drop(stopped);
    assert_eq!(stale_dropped.load(Ordering::Relaxed), 1);
    drop(second_image);
    runtime
        .snapshots()
        .release_snapshot(&selected_handle)
        .expect("selected owner snapshot releases after the witness");
}
