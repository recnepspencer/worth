use super::*;

#[path = "custody/state.rs"]
mod state;
pub(super) use state::RelationalRecoveryRecordState;

#[path = "custody/lease.rs"]
mod lease;
use lease::RelationalRecoveryLease;

impl ProductUnpublishedOwnerEffectsRecord {
    pub(crate) fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.identity
    }

    pub(crate) const fn metadata_bytes(&self) -> usize {
        self.metadata_bytes
    }

    pub(crate) fn settlement_required(&self) -> bool {
        self.progress.relational_requires_settlement()
    }

    /// The occurrence whose creation charged this record's custody, when the
    /// attempt created one. The catalog reads it as it releases the record, so
    /// the custody drain keys on what the record named and never on what a
    /// caller remembered.
    pub(crate) fn destination(&self) -> Option<(&ProductBranchIdentity, ProductBranchIncarnation)> {
        self.destination
            .as_ref()
            .map(|(branch, incarnation)| (branch, *incarnation))
    }

    pub(crate) fn next_actions(&self) -> &[ProductUnpublishedNextAction] {
        self.next_actions.as_slice()
    }

    pub(crate) fn take_relational_recovery(&mut self) -> Result<RelationalRecoveryLease<'_>, ()> {
        let progress = std::mem::replace(&mut self.progress, CompositeAttemptProgress::untouched());
        let component_results = std::mem::replace(
            &mut self.component_results,
            CompositeOwnerExecutionResults::retained(),
        );
        let (commit_identity, successor_basis, recovery_route, signal_posture) =
            match progress.into_relational_recovery_parts() {
                Ok(parts) => parts,
                Err(progress) => {
                    self.progress = progress;
                    self.component_results = component_results;
                    return Err(());
                }
            };
        let state = RelationalRecoveryRecordState::from_recovery_parts(
            commit_identity,
            successor_basis,
            recovery_route,
            component_results,
            signal_posture,
        );
        Ok(RelationalRecoveryLease::new(self, state))
    }

    pub(crate) fn restore_relational_recovery(&mut self, mut state: RelationalRecoveryRecordState) {
        let component_results = state
            .component_results
            .take()
            .expect("recovery result projection is restored exactly once");
        self.progress = state.into_progress();
        self.component_results = component_results;
    }

    pub(crate) fn settle_relational_recovery(
        &mut self,
        mut state: RelationalRecoveryRecordState,
        result: worth_relational::facade::transactions::CommitResult,
    ) {
        self.progress = state.settled_progress(result.clone());
        self.component_results = state
            .component_results
            .take()
            .expect("recovery result projection settles exactly once")
            .with_relational_settled(
                state.commit_identity.clone(),
                state.successor_basis.clone(),
                result,
            );
        state.route = None;
        self.next_actions = super::RetainedNextActions::from_vec(super::next_actions_for_progress(
            &self.progress,
            self.cause,
        ));
    }

    pub(crate) fn retain_pending_relational_settlement(
        &mut self,
        mut state: RelationalRecoveryRecordState,
        settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    ) {
        self.progress = state.pending_progress(settlement.clone());
        self.component_results = state
            .component_results
            .take()
            .expect("recovery result projection remains while settlement is pending")
            .with_relational_settlement_pending(
                state.commit_identity.clone(),
                state.successor_basis.clone(),
                settlement,
            );
        state.route = None;
        let next_actions = super::next_actions_for_progress(&self.progress, self.cause);
        // This record owes settlement: `recovery_continuation` reaches these
        // transitions only for a record whose Relational leg requires it, so
        // the derived continuation names settlement first.
        debug_assert_eq!(
            next_actions.first(),
            Some(&ProductUnpublishedNextAction::SettleOwnerEffects),
            "a retained record that owes settlement names settlement first"
        );
        self.next_actions = super::RetainedNextActions::from_vec(next_actions);
    }

    pub(crate) fn retain_identity_repair(&mut self, mut state: RelationalRecoveryRecordState) {
        self.progress = state.identity_required_progress();
        self.component_results = state
            .component_results
            .take()
            .expect("recovery result projection remains while identity repair is required");
        state.route = None;
        let next_actions = super::next_actions_for_progress(&self.progress, self.cause);
        // This record owes settlement: `recovery_continuation` reaches these
        // transitions only for a record whose Relational leg requires it, so
        // the derived continuation names settlement first.
        debug_assert_eq!(
            next_actions.first(),
            Some(&ProductUnpublishedNextAction::SettleOwnerEffects),
            "a retained record that owes settlement names settlement first"
        );
        self.next_actions = super::RetainedNextActions::from_vec(next_actions);
    }

    pub(crate) fn settle_relational_recovery_with_receipt(
        &mut self,
        mut state: RelationalRecoveryRecordState,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    ) {
        self.progress = state.settled_receipt_progress(receipt.clone());
        self.component_results = state
            .component_results
            .take()
            .expect("recovery result projection settles exactly once")
            .with_relational_settled_receipt(
                state.commit_identity.clone(),
                state.successor_basis.clone(),
                receipt,
            );
        state.route = None;
        self.next_actions = super::RetainedNextActions::from_vec(super::next_actions_for_progress(
            &self.progress,
            self.cause,
        ));
    }
}
