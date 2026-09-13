use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::retention::RetentionObligationDenial;

use super::{ActiveAttemptCustody, ActiveAttemptRecord, ActiveAttemptResources, ActivePinCustody};

/// Takes resources out of the record only while the caller capability is
/// borrowed. No World mutex survives acquisition or any other owner call.
/// Unwind restores the original resource custody before caller abandonment.
pub(crate) struct ActiveAttemptResourceLease<'a> {
    record: &'a ActiveAttemptRecord,
    resources: Option<ActiveAttemptResources>,
}

impl ActiveAttemptCustody {
    pub(crate) fn lease_resources(&mut self) -> ActiveAttemptResourceLease<'_> {
        let resources = self
            .record
            .state()
            .resources
            .take()
            .expect("only one phase leases attempt resources");
        ActiveAttemptResourceLease {
            record: &self.record,
            resources: Some(resources),
        }
    }

    pub(crate) fn bind_publication_pins(
        &mut self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<(), RetentionObligationDenial> {
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        match &mut resources.pins {
            ActivePinCustody::Reserved(capacity) => {
                let obligation = match capacity.try_bind_publication(basis) {
                    Ok(obligation) => obligation,
                    Err(denial) => {
                        resources.pin_denial = Some(denial.clone());
                        return Err(denial);
                    }
                };
                resources.pins = ActivePinCustody::Bound(obligation);
                resources.pin_denial = None;
            }
            ActivePinCustody::Bound(obligation) => assert!(obligation.matches_basis(basis)),
            ActivePinCustody::Retained { .. } => {
                unreachable!("retained custody cannot bind publication pins")
            }
            ActivePinCustody::TransferredToProduct => {
                unreachable!("performed custody cannot bind publication pins")
            }
        }
        if resources.history_pins.is_none() {
            let ActivePinCustody::Bound(pins) = &resources.pins else {
                unreachable!("binding completed")
            };
            resources.history_pins = Some(
                pins.fork_history(basis)
                    .map_err(RetentionObligationDenial::HistoryDependency)?,
            );
        }
        Ok(())
    }
}

impl ActiveAttemptResourceLease<'_> {
    pub(super) fn resources_mut(&mut self) -> &mut ActiveAttemptResources {
        self.resources
            .as_mut()
            .expect("an active lease owns its resources")
    }
}

impl Drop for ActiveAttemptResourceLease<'_> {
    fn drop(&mut self) {
        let Some(resources) = self.resources.take() else {
            return;
        };
        let mut state = self.record.state();
        assert!(
            state.resources.is_none(),
            "a resource lease restores exactly once"
        );
        state.resources = Some(resources);
    }
}
