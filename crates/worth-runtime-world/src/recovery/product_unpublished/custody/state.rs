use super::*;

pub(crate) struct RelationalRecoveryRecordState {
    pub(super) commit_identity: worth_relational::facade::history::RelationalCommitIdentity,
    pub(super) successor_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub(super) route: Option<RelationalRecoveryRecordRoute>,
    pub(super) component_results: Option<CompositeOwnerExecutionResults>,
    pub(super) signal_posture: crate::publication::SignalAttemptProgressPosture,
}

pub(super) enum RelationalRecoveryRecordRoute {
    Performed {
        performed: worth_relational::facade::mvcc::PerformedRelationalCommit,
    },
    SettlementPending {
        commit_identity: worth_relational::facade::history::RelationalCommitIdentity,
        successor_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    },
    IdentityRequired {
        commit_identity: worth_relational::facade::history::RelationalCommitIdentity,
        successor_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    },
}

impl RelationalRecoveryRecordRoute {
    fn settlement(
        &self,
    ) -> Option<&worth_relational::facade::publication::DeferredPublicationSettlement> {
        match self {
            Self::Performed { .. } => None,
            Self::SettlementPending { settlement, .. } => Some(settlement),
            Self::IdentityRequired { .. } => None,
        }
    }
}

impl RelationalRecoveryRecordState {
    pub(super) fn from_recovery_parts(
        commit_identity: worth_relational::facade::history::RelationalCommitIdentity,
        successor_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        recovery_route: crate::publication::RelationalRecoveryRoute,
        component_results: CompositeOwnerExecutionResults,
        signal_posture: crate::publication::SignalAttemptProgressPosture,
    ) -> Self {
        let route = match recovery_route {
            crate::publication::RelationalRecoveryRoute::Performed { performed } => {
                RelationalRecoveryRecordRoute::Performed { performed }
            }
            crate::publication::RelationalRecoveryRoute::SettlementPending { settlement } => {
                RelationalRecoveryRecordRoute::SettlementPending {
                    commit_identity: commit_identity.clone(),
                    successor_basis: successor_basis.clone(),
                    settlement,
                }
            }
            crate::publication::RelationalRecoveryRoute::IdentityRequired => {
                RelationalRecoveryRecordRoute::IdentityRequired {
                    commit_identity: commit_identity.clone(),
                    successor_basis: successor_basis.clone(),
                }
            }
        };
        Self {
            commit_identity,
            successor_basis,
            route: Some(route),
            component_results: Some(component_results),
            signal_posture,
        }
    }

    pub(crate) fn commit_identity(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitIdentity {
        &self.commit_identity
    }

    pub(crate) fn settlement(
        &self,
    ) -> Option<&worth_relational::facade::publication::DeferredPublicationSettlement> {
        self.route
            .as_ref()
            .and_then(RelationalRecoveryRecordRoute::settlement)
    }

    pub(crate) fn take_performed(
        &mut self,
    ) -> Option<worth_relational::facade::mvcc::PerformedRelationalCommit> {
        let route = self.route.take()?;
        match route {
            RelationalRecoveryRecordRoute::Performed { performed, .. } => Some(performed),
            route => {
                self.route = Some(route);
                None
            }
        }
    }

    pub(crate) fn take_identity_repair(
        &mut self,
    ) -> Option<worth_relational::facade::history::RelationalCommitIdentity> {
        let route = self.route.take()?;
        match route {
            RelationalRecoveryRecordRoute::IdentityRequired {
                commit_identity, ..
            } => Some(commit_identity),
            route => {
                self.route = Some(route);
                None
            }
        }
    }

    pub(crate) fn restore_identity_repair(&mut self) {
        assert!(self.route.is_none(), "identity repair is restored once");
        self.route = Some(RelationalRecoveryRecordRoute::IdentityRequired {
            commit_identity: self.commit_identity.clone(),
            successor_basis: self.successor_basis.clone(),
        });
    }

    pub(super) fn into_progress(mut self) -> CompositeAttemptProgress {
        let signal_posture = self.signal_posture;
        match self
            .route
            .take()
            .expect("a recoverable route is restored exactly once")
        {
            RelationalRecoveryRecordRoute::Performed { performed, .. } => {
                CompositeAttemptProgress::new(
                    crate::publication::RelationalAttemptProgress::performed(performed),
                    crate::publication::SignalAttemptProgress::summary(signal_posture),
                )
            }
            RelationalRecoveryRecordRoute::SettlementPending {
                commit_identity,
                successor_basis,
                settlement,
            } => CompositeAttemptProgress::new(
                crate::publication::RelationalAttemptProgress::settlement_pending(
                    commit_identity,
                    successor_basis,
                    settlement,
                ),
                crate::publication::SignalAttemptProgress::summary(signal_posture),
            ),
            RelationalRecoveryRecordRoute::IdentityRequired {
                commit_identity,
                successor_basis,
            } => CompositeAttemptProgress::new(
                crate::publication::RelationalAttemptProgress::settlement_required(
                    commit_identity,
                    successor_basis,
                ),
                crate::publication::SignalAttemptProgress::summary(signal_posture),
            ),
        }
    }

    pub(super) fn settled_progress(
        &self,
        result: worth_relational::facade::transactions::CommitResult,
    ) -> CompositeAttemptProgress {
        let progress = crate::publication::RelationalAttemptProgress::settled(
            self.commit_identity.clone(),
            self.successor_basis.clone(),
            result,
        );
        CompositeAttemptProgress::new(
            progress,
            crate::publication::SignalAttemptProgress::summary(self.signal_posture),
        )
    }

    pub(super) fn pending_progress(
        &self,
        settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    ) -> CompositeAttemptProgress {
        let progress = crate::publication::RelationalAttemptProgress::settlement_pending(
            self.commit_identity.clone(),
            self.successor_basis.clone(),
            settlement,
        );
        CompositeAttemptProgress::new(
            progress,
            crate::publication::SignalAttemptProgress::summary(self.signal_posture),
        )
    }

    pub(super) fn identity_required_progress(&self) -> CompositeAttemptProgress {
        let progress = crate::publication::RelationalAttemptProgress::settlement_required(
            self.commit_identity.clone(),
            self.successor_basis.clone(),
        );
        CompositeAttemptProgress::new(
            progress,
            crate::publication::SignalAttemptProgress::summary(self.signal_posture),
        )
    }

    pub(super) fn settled_receipt_progress(
        &self,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    ) -> CompositeAttemptProgress {
        let progress = crate::publication::RelationalAttemptProgress::settled_receipt(
            self.commit_identity.clone(),
            self.successor_basis.clone(),
            receipt,
        );
        CompositeAttemptProgress::new(
            progress,
            crate::publication::SignalAttemptProgress::summary(self.signal_posture),
        )
    }
}
