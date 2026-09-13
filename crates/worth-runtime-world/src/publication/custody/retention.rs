use std::sync::Arc;

use crate::branch::ProductBranchReferenceSnapshot;
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedOwnerEffects};

use super::{ActiveAttemptCustody, ActiveHistoryCustody, ActivePinCustody};

/// A pre-movement loser releases the unused history slot. An intermediate
/// owner-effect terminal installs its already prepared successor for recovery.
pub(crate) enum RetainedCommitDisposition {
    InstallSuccessor,
    ReleaseUnused,
}

impl ActiveAttemptCustody {
    /// Complete an explicit retained terminal while the owner record keeps
    /// custody across every history operation, pin acquisition, and retag.
    pub(crate) fn retain(
        mut self,
        cause: ProductUnpublishedCause,
        observed: Option<ProductBranchReferenceSnapshot>,
        disposition: RetainedCommitDisposition,
    ) -> ProductUnpublishedOwnerEffects {
        {
            let mut state = self.record.state();
            state.cause = cause;
            state.last_observed = observed;
        }
        self.prepare_retained_resources(disposition);
        self.begin_recovery();
        #[cfg(test)]
        if self.record.state().destination.is_some() {
            super::creation_rehearsal::pause_before_forked_recovery_record(self.record.identity());
        }
        let slot = self
            .slot
            .take()
            .expect("an active caller holds its recovery slot");
        slot.retain_active(self.record.identity())
    }

    fn prepare_retained_resources(&mut self, disposition: RetainedCommitDisposition) {
        self.retain_creation_resources();
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        drop(resources.prepared_successor_observation.take());
        drop(resources.successor_observation_capacity.take());
        drop(resources.successor_observation_pins.take());
        let commit = Arc::clone(
            resources
                .commit
                .as_ref()
                .expect("explicit retention has prepared its successor"),
        );
        if let Some(head) = resources.product_head.as_mut() {
            let retained = head
                .retain_component_pins()
                .expect("the exact returned head pair retains atomically");
            resources.pins = ActivePinCustody::Retained {
                _obligation: retained,
            };
            let (_, _, history, _) = resources
                .product_head
                .take()
                .expect("head custody was just borrowed")
                .into_parts();
            resources.history_custody = ActiveHistoryCustody::Installed(history);
        }
        if resources.pin_denial.is_none() {
            if let ActivePinCustody::Reserved(capacity) = &mut resources.pins {
                match capacity.try_bind_publication(commit.basis()) {
                    Ok(obligation) => resources.pins = ActivePinCustody::Bound(obligation),
                    Err(denial) => resources.pin_denial = Some(denial),
                }
            }
        }
        match disposition {
            RetainedCommitDisposition::InstallSuccessor => {
                if let ActiveHistoryCustody::Reserved(capacity) = &mut resources.history_custody {
                    if resources.history_pins.is_none() {
                        if let ActivePinCustody::Bound(pins) = &resources.pins {
                            match pins.fork_history(commit.basis()) {
                                Ok(history) => resources.history_pins = Some(history),
                                Err(denial) => resources.pin_denial = Some(
                                    crate::retention::RetentionObligationDenial::HistoryDependency(
                                        denial,
                                    ),
                                ),
                            }
                        }
                    }
                    // A denied pin binding retains the original reserved slot;
                    // it never installs an unpinned history occurrence.
                    if let Some(pins) = resources.history_pins.take() {
                        let protection = capacity
                            .try_install_product_head(Arc::clone(&commit), pins)
                            .expect("reserved successor installs with exact history pins");
                        resources.history_custody = ActiveHistoryCustody::Installed(protection);
                    }
                }
            }
            RetainedCommitDisposition::ReleaseUnused => {
                if matches!(resources.history_custody, ActiveHistoryCustody::Reserved(_)) {
                    resources.history_custody = ActiveHistoryCustody::Released;
                    resources.commit = None;
                    resources.history_pins = None;
                }
            }
        }
        if let ActivePinCustody::Bound(obligation) = &mut resources.pins {
            let retained = obligation
                .try_transfer_retained()
                .expect("live publication claims transfer together into retained custody");
            resources.pins = ActivePinCustody::Retained {
                _obligation: retained,
            };
        }
    }
}
