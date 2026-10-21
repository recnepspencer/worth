//! One full verification at the live source image re-establishes its row as
//! marked-clean there, so the next demand reads marks instead of verifying again.

use std::sync::Arc;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::RecordedSettlementIdentity;
use super::super::SealedNativeOutputWitness;
use super::mark_state::FullVerificationReason;
use super::mark_state::SettlementCurrentness;
use super::source_alignment::EqualOutputCurrentness;
use super::source_alignment::SnapshotAlignedMarkState;
use super::verification::SettlementVerificationStop;
use super::SettlementRegistration;
use super::SettlementRegistrationStop;
use super::SourceInvalidationOwner;

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
        facts: &super::super::ComparableSourceFacts,
        witness: &SealedNativeOutputWitness,
    ) -> Result<bool, SettlementRegistrationStop> {
        let Some(cell) = self.cell_for_read(read_basis)? else {
            return Ok(false);
        };
        let image = cell.read_image();

        if image.payload().current.settlements.contains_key(identity) {
            return Ok(true);
        }
        drop(image);
        let Some(output_facts) = self.prepare_performed_output_facts(witness)? else {
            return Ok(false);
        };
        self.register_settlement(SettlementRegistration {
            computation: None,
            work_membership: None,
            identity: Arc::clone(identity),
            facts: facts.retained(),
            output_facts: Some(output_facts),
            read_basis: read_basis.clone(),
            stale_at_read_basis: im::OrdSet::new(),
            requirement: None,
            upstream: im::OrdSet::new(),
        })?;
        Ok(true)
    }

    /// A fresh consumer compared this missing upstream's original sources and
    /// sealed output after its own effect. Its upstream rows were established
    /// first; their exact identities are preserved in this new posting.
    pub(in crate::domain_computation::primary_graph) fn establish_verified_consumed(
        &self,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        facts: &super::super::ComparableSourceFacts,
        witness: &SealedNativeOutputWitness,
        upstream: im::OrdSet<Arc<RecordedSettlementIdentity>>,
    ) -> Result<(), SettlementRegistrationStop> {
        let output_facts = self
            .prepare_performed_output_facts(witness)?
            .ok_or(SettlementRegistrationStop::SourceUnavailable)?;
        self.register_settlement(SettlementRegistration {
            computation: None,
            work_membership: None,
            identity: Arc::clone(identity),
            facts: facts.retained(),
            output_facts: Some(output_facts),
            read_basis: selected.clone(),
            stale_at_read_basis: im::OrdSet::new(),
            requirement: None,
            upstream,
        })
    }

    /// The caller fully compared this row's original source facts and its
    /// registered output facts or sealed witness at `selected`. For a
    /// delivery-only gap, that proof and clean consumed upstreams supersede
    /// both dirty ordinals and pending edges. Incomplete postings or unclean
    /// upstreams keep requiring verification.
    /// Returns whether the row is marked-clean at `selected`.
    pub(in crate::domain_computation::primary_graph) fn reestablish_verified(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        verified_facts: &super::super::ComparableSourceFacts,
    ) -> Result<bool, SettlementVerificationStop> {
        let actual = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment);
        }
        let Some(cell) = self.cell_for_read(selected)? else {
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

        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(|_| SettlementVerificationStop::Alignment)?;

        let Some(current_row) = image.payload().current.settlements.get(identity) else {
            return Ok(false);
        };
        if current_row.facts.for_comparison().is_none() {
            return Ok(false);
        }
        match aligned.currentness(identity) {
            SettlementCurrentness::Clean => {
                // Compared in full here, so the clean row is verified through
                // this image as well.
                drop(aligned);
                drop(image);
                let row = std::slice::from_ref(identity);
                self.carry_read_basis(runtime, snapshot, selected, row)?;
                return Ok(true);
            }
            SettlementCurrentness::FullVerificationRequired(_) => {}
            // A row read in another runtime is never re-established here.
            SettlementCurrentness::Foreign => return Err(SettlementVerificationStop::Alignment),
            // Exact in-window dirty/pending rows use their own reverification;
            // only a fully compared delivery gap follows the recovery below.
            SettlementCurrentness::Dirty(_) | SettlementCurrentness::PendingUpstream(_) => {
                return Ok(false);
            }
        }
        let state = &image.payload().current;

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
            || !row
                .facts
                .for_comparison()
                .is_some_and(|facts| facts.facts().same_backing(verified_facts.facts()))
        {
            return Ok(false);
        }
        for upstream in &row.consumed_upstream {
            let equal = aligned.equal_output_currentness(upstream)?;
            let clean = match equal {
                EqualOutputCurrentness::CanonicallyEqualClean(_) => true,
                EqualOutputCurrentness::NoConsequence => {
                    matches!(aligned.currentness(upstream), SettlementCurrentness::Clean)
                }
                EqualOutputCurrentness::Pending
                | EqualOutputCurrentness::FullVerificationRequired => false,
            };
            if !clean {
                return Ok(false);
            }
        }

        let mut replacement = (**row).clone();
        // Ordinary full verification cannot requalify original computation coverage.
        replacement.computation_frontier = None;
        replacement.dirty_ordinals = im::OrdSet::new();
        replacement.pending_upstream = im::OrdSet::new();
        replacement.read_basis = Arc::new(selected.clone());
        replacement.verification_requirement = None;
        let mut next = (**state).clone();

        replacement.delivery_epoch = next.delivery_epoch;
        next.computation_dirty_ordinal_count -=
            row.computation_frontier.as_ref().map_or(0, im::OrdSet::len);
        next.dirty_ordinal_count -= row.dirty_ordinals.len();
        next.pending_edge_count -= row.pending_upstream.len();

        next.settlements
            .insert(Arc::clone(identity), Arc::new(replacement));
        self.install_live_state(cell, image, next)?;
        Ok(true)
    }
}
