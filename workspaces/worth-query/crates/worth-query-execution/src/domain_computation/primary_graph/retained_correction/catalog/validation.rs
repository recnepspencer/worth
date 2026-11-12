//! Validate complete semantic transitions before any fold mutation or publication.
use super::*;
impl MaterialCatalog {
    pub(in crate::domain_computation::primary_graph) fn validate(
        &self,
        record: &MaterialRecord,
        position: u64,
    ) -> Result<(), Denial> {
        match record {
            MaterialRecord::RecoveryPin {
                identity, original, ..
            } => {
                self.admit_original(*original)?;
                if identity.home() != self.home || identity.acquire_position().get() != position {
                    return Err(Denial::OriginalMismatch);
                }
                self.material
                    .get(original)
                    .ok_or(Denial::Unavailable)?
                    .available()?;
                if self.pins.contains_key(identity)
                    || self.pins.values().any(|row| {
                        row.owner
                            == (CorrectionMaterialPinOwner::Recovery {
                                original: *original,
                            })
                    })
                {
                    return Err(Denial::PinAlreadyClaimed);
                }
            }
            MaterialRecord::Capture { material } => {
                self.admit_original(material.original)?;
                if self.material.contains_key(&material.original) {
                    return Err(Denial::MaterialAlreadyCaptured);
                }
            }
            MaterialRecord::Ready { material } => {
                self.admit_original(material.original)?;
                let entry = self
                    .material
                    .get(&material.original)
                    .ok_or(Denial::Unavailable)?;
                match &entry.body {
                    MaterialBody::Captured(_) => {}
                    MaterialBody::Ready { .. }
                    | MaterialBody::Expired { .. }
                    | MaterialBody::Abandoned => return Err(Denial::MaterialAlreadyCaptured),
                }
            }
            MaterialRecord::Pin {
                identity,
                original,
                owner,
                prepared,
                ..
            } => {
                self.admit_original(*original)?;
                self.admit_owner(*owner)?;
                if identity.home() != self.home || identity.acquire_position().get() != position {
                    return Err(Denial::OriginalMismatch);
                }
                self.material
                    .get(original)
                    .ok_or(Denial::Unavailable)?
                    .available()?;
                match owner {
                    CorrectionMaterialPinOwner::PreparedCommand { key } if *key == prepared.key => {
                    }
                    CorrectionMaterialPinOwner::PreparedCommand { .. }
                    | CorrectionMaterialPinOwner::Publication { .. }
                    | CorrectionMaterialPinOwner::Recovery { .. } => {
                        return Err(Denial::PinOwnerMismatch)
                    }
                }
                if prepared.target.owner.home() != self.home {
                    return Err(Denial::ForeignHome);
                }
                if self.discharged.contains_key(&prepared.key) {
                    return Err(Denial::PreparationDischarged);
                }
                if self.pins.contains_key(identity)
                    || self
                        .pins
                        .values()
                        .any(|row| row.original == *original && row.owner == *owner)
                {
                    return Err(Denial::PinAlreadyClaimed);
                }
            }
            MaterialRecord::Transfer {
                identity,
                before,
                after,
                ..
            } => {
                self.admit_owner(*after)?;
                if self
                    .pins
                    .iter()
                    .any(|(other, row)| other != identity && row.owner == *after)
                {
                    return Err(Denial::PinAlreadyClaimed);
                }
                let current = self.pins.get(identity).ok_or(Denial::UnknownPin)?;
                if current.owner != *before {
                    return Err(Denial::PinOwnerMismatch);
                }
                match (before, after) {
                    (
                        CorrectionMaterialPinOwner::PreparedCommand { key },
                        CorrectionMaterialPinOwner::Publication { .. },
                    ) if *key == current.prepared()?.key => {}
                    (
                        CorrectionMaterialPinOwner::Recovery { .. },
                        CorrectionMaterialPinOwner::Recovery { .. }
                        | CorrectionMaterialPinOwner::PreparedCommand { .. }
                        | CorrectionMaterialPinOwner::Publication { .. },
                    )
                    | (
                        CorrectionMaterialPinOwner::PreparedCommand { .. }
                        | CorrectionMaterialPinOwner::Publication { .. },
                        CorrectionMaterialPinOwner::Recovery { .. },
                    )
                    | (
                        CorrectionMaterialPinOwner::PreparedCommand { .. },
                        CorrectionMaterialPinOwner::PreparedCommand { .. },
                    )
                    | (
                        CorrectionMaterialPinOwner::PreparedCommand { .. },
                        CorrectionMaterialPinOwner::Publication { .. },
                    )
                    | (
                        CorrectionMaterialPinOwner::Publication { .. },
                        CorrectionMaterialPinOwner::PreparedCommand { .. },
                    )
                    | (
                        CorrectionMaterialPinOwner::Publication { .. },
                        CorrectionMaterialPinOwner::Publication { .. },
                    ) => return Err(Denial::PinOwnerMismatch),
                }
            }
            MaterialRecord::Release {
                identity, owner, ..
            } => {
                if let CorrectionMaterialPinOwner::Recovery { original } = owner {
                    let pin = self.pins.get(identity).ok_or(Denial::UnknownPin)?;
                    if pin.owner != *owner || pin.original != *original {
                        return Err(Denial::PinOwnerMismatch);
                    }
                    self.material
                        .get(original)
                        .ok_or(Denial::MaterialNotReady)?
                        .available()?;
                    return Ok(());
                }
                match owner {
                    CorrectionMaterialPinOwner::Publication { .. } => {}
                    CorrectionMaterialPinOwner::Recovery { .. } => {
                        unreachable!("handled recovery release")
                    }
                    CorrectionMaterialPinOwner::PreparedCommand { .. } => {
                        return Err(Denial::PinOwnerMismatch)
                    }
                }
                let pin = self.pins.get(identity).ok_or(Denial::UnknownPin)?;
                if pin.owner != *owner {
                    return Err(Denial::PinOwnerMismatch);
                }
                let CorrectionMaterialPinOwner::Publication { original } = owner else {
                    unreachable!("publication release checked above")
                };
                let material = self
                    .material
                    .get(original)
                    .ok_or(Denial::MaterialNotReady)?;
                material.available()?;
                let intent = crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding::observe_recorded_intent(&material.metadata.intent)
                    .map_err(|_| Denial::OriginalBindingUnavailable)?;
                if material.metadata.key != pin.prepared()?.key
                    || material.metadata.input_identity != pin.prepared()?.input_identity
                    || intent.correction != Some(pin.prepared()?.intent)
                {
                    return Err(Denial::PinIntentDrift);
                }
            }
            MaterialRecord::DischargePreparation { identity, key, .. } => {
                let pin = self.pins.get(identity).ok_or(Denial::UnknownPin)?;
                if pin.owner != (CorrectionMaterialPinOwner::PreparedCommand { key: *key }) {
                    return Err(Denial::PinOwnerMismatch);
                }
                if self.discharged.contains_key(key) {
                    return Err(Denial::PreparationDischarged);
                }
            }
            MaterialRecord::Abandon { original } => {
                // A released unpublished owner may discharge its own publication
                // pin. It may never release another command's use of its material.
                if self.pins.values().any(|pin| pin.original == *original) {
                    return Err(Denial::MaterialPinned);
                }
                match &self.material.get(original).ok_or(Denial::Unavailable)?.body {
                    MaterialBody::Captured(_) => {}
                    MaterialBody::Ready { .. }
                    | MaterialBody::Expired { .. }
                    | MaterialBody::Abandoned => return Err(Denial::OriginalMismatch),
                }
            }
            MaterialRecord::Expire { original } => {
                if self.pin_count(*original) != 0 {
                    return Err(Denial::MaterialPinned);
                }
                self.material
                    .get(original)
                    .ok_or(Denial::Unavailable)?
                    .available()?;
            }
        }
        Ok(())
    }
    fn admit_original(&self, original: Original) -> Result<(), Denial> {
        if original.owner().home() == self.home {
            Ok(())
        } else {
            Err(Denial::ForeignHome)
        }
    }
    fn admit_owner(&self, owner: CorrectionMaterialPinOwner) -> Result<(), Denial> {
        match owner {
            CorrectionMaterialPinOwner::PreparedCommand { .. } => Ok(()),
            CorrectionMaterialPinOwner::Publication { original }
            | CorrectionMaterialPinOwner::Recovery { original } => self.admit_original(original),
        }
    }
}
