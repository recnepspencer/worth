//! One full verification at the live source image re-establishes its row as
//! marked-clean there, so the next demand reads marks instead of verifying again.

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

use super::super::{RecordedSettlementIdentity, SealedNativeOutputWitness};
use super::{
    admission::IndexAdmission,
    index_capacity,
    mark_state::{FullVerificationReason, MarkState, SettlementCurrentness, SettlementMarks},
    source_alignment::{EqualOutputCurrentness, SnapshotAlignedMarkState},
    verification::SettlementVerificationStop,
    InvalidationEditAdmission, SettlementRegistration, SettlementRegistrationStop,
    SourceInvalidationOwner,
};

impl SourceInvalidationOwner {
    /// A restored output has no row until it is verified on this runtime. It
    /// consumed nothing, so its source facts and output witness, compared in
    /// full at `read_basis`, are all its row needs; deliveries retained after
    /// that basis are replayed onto it. An output that already has a row keeps
    /// it. Returns whether the output has a row.
    pub(in crate::domain_computation::primary_graph) fn establish_verified_root(
        &self,
        read_basis: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        facts: &Arc<[WorthQueryApplicationObservedFact]>,
        witness: &SealedNativeOutputWitness,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, SettlementRegistrationStop> {
        let Some(cell) = self.cell_for_read(read_basis, admission)? else {
            return Ok(false);
        };
        let image = cell.read_image();
        admission.ordered_read(image.payload().current.settlements.len())?;
        if image.payload().current.settlements.contains_key(identity) {
            return Ok(true);
        }
        drop(image);
        let Some(output_facts) = self.prepare_performed_output_facts(witness, admission)? else {
            return Ok(false);
        };
        self.register_settlement(
            SettlementRegistration {
                work_membership: None,
                identity: Arc::clone(identity),
                facts: Arc::clone(facts),
                output_facts: Some(output_facts),
                read_basis: read_basis.clone(),
                stale_at_read_basis: im::OrdSet::new(),
                requirement: None,
                upstream: im::OrdSet::new(),
            },
            admission,
        )?;
        Ok(true)
    }

    /// The caller compared every source fact and the output witness of this
    /// row at `selected`. That proof replaces whatever delivery the row could
    /// not account for: its read basis moves to the live image and its epoch
    /// to the live one. A row whose postings are incomplete, or whose consumed
    /// outputs are not themselves clean, keeps requiring verification.
    /// Returns whether the row is marked-clean at `selected`.
    pub(in crate::domain_computation::primary_graph) fn reestablish_verified(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        verified_facts: &[WorthQueryApplicationObservedFact],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, SettlementVerificationStop> {
        let branch_bytes = selected.branch_id().0.len() as u64;
        admission.work(1 + branch_bytes)?;
        admission.bytes(branch_bytes)?;
        let actual = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment);
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            return Ok(false);
        };
        let image = cell.read_image();
        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            // Historical states stay immutable.
            return Ok(false);
        }
        admission.ordered_read(image.payload().past.len())?;
        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        admission.ordered_read(aligned.settlement_count())?;
        match aligned.currentness(identity) {
            SettlementCurrentness::Clean => {
                // Compared in full here, so the clean row is verified through
                // this image as well.
                drop(aligned);
                drop(image);
                let row = std::slice::from_ref(identity);
                self.carry_read_basis(runtime, snapshot, selected, row, admission)?;
                return Ok(true);
            }
            SettlementCurrentness::FullVerificationRequired(_) => {}
            // A row read in another runtime is never re-established here.
            SettlementCurrentness::Foreign => return Err(SettlementVerificationStop::Alignment),
            // Dirty and pending rows keep their own exact reverification.
            SettlementCurrentness::Dirty(_) | SettlementCurrentness::PendingUpstream(_) => {
                return Ok(false)
            }
        }
        let state = &image.payload().current;
        admission.ordered_read(state.settlements.len())?;
        let Some(row) = state.settlements.get(identity) else {
            return Ok(false);
        };
        let delivery_only = matches!(
            row.verification_requirement,
            None | Some(
                FullVerificationReason::RetainedDeliveryGap
                    | FullVerificationReason::DeclaredChangeUnavailable
            )
        );
        if !delivery_only
            || !row.pending_upstream.is_empty()
            || !std::ptr::eq(row.facts.as_ref(), verified_facts)
        {
            return Ok(false);
        }
        for upstream in &row.consumed_upstream {
            admission.work(1)?;
            let equal = aligned.equal_output_currentness(upstream, admission)?;
            let clean = match equal {
                EqualOutputCurrentness::CanonicallyEqualClean(_) => true,
                EqualOutputCurrentness::NoConsequence => {
                    admission.ordered_read(aligned.settlement_count())?;
                    matches!(aligned.currentness(upstream), SettlementCurrentness::Clean)
                }
                EqualOutputCurrentness::Pending
                | EqualOutputCurrentness::FullVerificationRequired => false,
            };
            if !clean {
                return Ok(false);
            }
        }

        let before = admission.charged_bytes();
        admission.bytes(
            index_capacity::arc_bytes::<SettlementMarks>()
                .and_then(|bytes| {
                    bytes.checked_add(index_capacity::arc_bytes::<PositionedRelationalSnapshot>()?)
                })
                .and_then(|bytes| bytes.checked_add(index_capacity::arc_bytes::<MarkState>()?))
                .and_then(|bytes| bytes.checked_add(branch_bytes))
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut replacement = (**row).clone();
        replacement.dirty_ordinals = im::OrdSet::new();
        replacement.read_basis = Arc::new(selected.clone());
        replacement.verification_requirement = None;
        let mut next = (**state).clone();
        replacement.delivery_epoch = next.delivery_epoch;
        next.dirty_ordinal_count -= row.dirty_ordinals.len();
        admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
            next.settlements.len(),
        )?;
        next.settlements
            .insert(Arc::clone(identity), Arc::new(replacement));
        self.install_live_state(cell, image, (next, before), admission)?;
        Ok(true)
    }
}
