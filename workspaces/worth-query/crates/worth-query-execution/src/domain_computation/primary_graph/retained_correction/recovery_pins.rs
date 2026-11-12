//! Mandatory material uses belong to original recovery bindings, not commands.
use super::*;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity as Identity, CorrectionMaterialPinOwner as Owner,
    DurableCompositePublicationAttemptIdentity as Original, RetentionClass,
};

impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn acquire_recovery_pin(
        &self,
        original: Original,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Identity, Denial> {
        let driver = self.publication()?;
        driver.finish_pending(self.medium()?.as_ref(), request)?;
        let identity = {
            let state = self.state();
            state
                .catalog
                .material
                .get(&original)
                .ok_or(Denial::Unavailable)?
                .available()?;
            if let Some((identity, _)) = state
                .catalog
                .pins
                .iter()
                .find(|(_, pin)| pin.owner == (Owner::Recovery { original }))
            {
                return if state.pins.contains(*identity) {
                    Ok(*identity)
                } else {
                    Err(Denial::OwnerUnavailable)
                };
            }
            Identity::from_recorded(
                state.catalog.home,
                state
                    .catalog
                    .position
                    .checked_add(1)
                    .and_then(std::num::NonZeroU64::new)
                    .ok_or(Denial::IdentityExhausted)?,
            )
        };
        driver.publish(
            MaterialRecord::RecoveryPin {
                identity,
                original,
                class: RetentionClass::MandatoryObligation,
            },
            request,
        )?;
        Ok(identity)
    }
    pub(in crate::domain_computation::primary_graph) fn release_recovery_pin(
        &self,
        identity: Identity,
        original: Original,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let driver = self.publication()?;
        driver.finish_pending(self.medium()?.as_ref(), request)?;
        let owner = Owner::Recovery { original };
        {
            let state = self.state();
            let Some(pin) = state.catalog.pins.get(&identity) else {
                return if state.catalog.released.get(&identity) == Some(&owner) {
                    Ok(())
                } else {
                    Err(Denial::UnknownPin)
                };
            };
            if pin.owner != owner || pin.original != original {
                return Err(Denial::PinOwnerMismatch);
            }
        }
        driver.publish(
            MaterialRecord::Release {
                identity,
                owner,
                class: RetentionClass::OptionalRetainedMaterial,
            },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn settled_recovery_pins(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Vec<(Identity, Original)>, Denial> {
        self.settle(request)?;
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        Ok(state
            .catalog
            .pins
            .iter()
            .filter_map(|(identity, row)| match row.owner {
                Owner::Recovery { original } => Some((*identity, original)),
                Owner::PreparedCommand { .. } | Owner::Publication { .. } => None,
            })
            .collect())
    }
}
