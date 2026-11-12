//! Receiver-owned command custody; retained callers carry only its address.
use super::record::PreparedCorrectionSnapshot;
use super::*;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity as Identity, CorrectionMaterialPinOwner as Owner,
    DurableCompositePublicationAttemptIdentity as Original, RetentionClass,
};

impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn acquire_prepared(
        &self,
        original: Original,
        prepared: PreparedCorrectionSnapshot,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Identity, Denial> {
        let driver = self.publication()?;
        driver.finish_pending(self.medium()?.as_ref(), request)?;
        let identity = {
            let state = self.state();
            if state.catalog.discharged.contains_key(&prepared.key) {
                return Err(Denial::PreparationDischarged);
            }
            if let Some((identity, pin)) = state
                .catalog
                .pins
                .iter()
                .find(|(_, pin)| pin.prepared.key == prepared.key)
            {
                if pin.original != original || *pin.prepared != prepared {
                    return Err(Denial::PinIntentDrift);
                }
                match pin.owner {
                    Owner::PreparedCommand { .. } if state.pins.contains(*identity) => {
                        return Ok(*identity)
                    }
                    Owner::PreparedCommand { .. } => return Err(Denial::OwnerUnavailable),
                    Owner::Publication { .. } => return Err(Denial::PublicationPending),
                }
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
        let key = prepared.key;
        driver.publish(
            MaterialRecord::Pin {
                identity,
                original,
                owner: Owner::PreparedCommand { key },
                prepared: Box::new(prepared),
                class: RetentionClass::MandatoryObligation,
            },
            request,
        )?;
        Ok(identity)
    }
    pub(in crate::domain_computation::primary_graph) fn prepared_command(
        &self,
        original: Original,
        key: [u8; 32],
    ) -> Result<Option<(Identity, Arc<PreparedCorrectionSnapshot>)>, Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        if state.catalog.discharged.contains_key(&key) {
            return Err(Denial::PreparationDischarged);
        }
        let Some((identity, pin)) = state
            .catalog
            .pins
            .iter()
            .find(|(_, pin)| pin.prepared.key == key)
        else {
            return Ok(None);
        };
        if pin.original != original {
            return Err(Denial::PinIntentDrift);
        }
        match pin.owner {
            Owner::PreparedCommand { .. } if state.pins.contains(*identity) => {
                Ok(Some((*identity, Arc::clone(&pin.prepared))))
            }
            Owner::PreparedCommand { .. } => Err(Denial::OwnerUnavailable),
            Owner::Publication { .. } => Err(Denial::PublicationPending),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn transfer_prepared(
        &self,
        identity: Identity,
        opened: &worth_runtime_world::facade::WorldOpenedPublicationAttempt,
        authority: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let driver = self.publication()?;
        driver.finish_pending(self.medium()?.as_ref(), request)?;
        let after = Owner::Publication {
            original: opened.association().attempt(),
        };
        let before = {
            let state = self.state();
            let pin = state
                .catalog
                .pins
                .get(&identity)
                .ok_or(Denial::UnknownPin)?;
            let snapshot = &pin.prepared;
            let expected = opened.expected_head();
            let source = expected.basis().relational_basis().observation();
            let input = authority
                .retained_governed_input_carrier()
                .and_then(|input| input.material_input())
                .ok_or(Denial::OriginalMismatch)?;
            let principal = authority.principal_scope().principal();
            let scope = authority.principal_scope().scope();
            if *authority.idempotency_binding().key_identity() != snapshot.key
                || authority.idempotency_binding().correction_identity() != Some(&snapshot.intent)
                || input.1 != snapshot.action
                || input.2 != snapshot.input_identity
                || input.0.recorded_bytes() != snapshot.input.as_slice()
                || principal.partition_id() != snapshot.actor.partition_value()
                || principal.local_slot() != snapshot.actor.local_slot_value()
                || principal.generation() != snapshot.actor.generation_value()
                || scope.partition_id() != snapshot.scope.subject.partition_value()
                || scope.local_slot() != snapshot.scope.subject.local_slot_value()
                || scope.generation() != snapshot.scope.subject.generation_value()
            {
                return Err(Denial::PinOwnerMismatch);
            }
            if expected.owner_identity().durable() != Some(snapshot.target.owner)
                || expected.branch_identity().name().as_str() != snapshot.target.branch
                || expected.lifecycle_incarnation().ordinal() != snapshot.target.incarnation
                || expected.reference_generation().get() != snapshot.target.generation
                || expected.selected_commit().ordinal() != snapshot.target.commit
                || source.commit_receipt() != Some(&snapshot.target.relational)
                || opened.program_revision().bytes() != &snapshot.target.program
            {
                return Err(Denial::TargetChanged);
            }
            if pin.owner == after {
                return Ok(());
            }
            pin.owner
        };
        driver.publish(
            MaterialRecord::Transfer {
                identity,
                before,
                after,
                class: RetentionClass::MandatoryObligation,
            },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn release_publication(
        &self,
        identity: Identity,
        original: Original,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let driver = self.publication()?;
        driver.finish_pending(self.medium()?.as_ref(), request)?;
        {
            let state = self.state();
            let Some(pin) = state.catalog.pins.get(&identity) else {
                return if state.catalog.released.get(&identity)
                    == Some(&Owner::Publication { original })
                {
                    Ok(())
                } else {
                    Err(Denial::UnknownPin)
                };
            };
            if pin.owner != (Owner::Publication { original }) {
                return Err(Denial::PinOwnerMismatch);
            }
            let published = state
                .catalog
                .material
                .get(&original)
                .ok_or(Denial::MaterialNotReady)?;
            published.available()?;
            if published.metadata.key != pin.prepared.key {
                return Err(Denial::OriginalMismatch);
            }
        }
        driver.publish(
            MaterialRecord::Release {
                identity,
                owner: Owner::Publication { original },
                class: RetentionClass::OptionalRetainedMaterial,
            },
            request,
        )
    }
}
