//! Idempotent semantic publication by the original acknowledged owner.
use super::catalog::MaterialBody;
use super::record::{MaterialCapture, MaterialReady};
use super::*;
impl CorrectionMaterialOwner {
    pub fn abandon_released(
        &self,
        originals: &worth_runtime_world::facade::WorldProgramActivationHistory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.settle(request)?;
        let released = self
            .state()
            .catalog
            .material
            .iter()
            .filter_map(|((identity, _), material)| match &material.body {
                MaterialBody::Captured(_)
                    if originals.original_state(*identity).is_some_and(|original| {
                        original.fate() == worth_foundational::facade::WorldOriginalFate::Released
                    }) =>
                {
                    Some(*identity)
                }
                MaterialBody::Captured(_)
                | MaterialBody::Ready { .. }
                | MaterialBody::Expired { .. }
                | MaterialBody::Abandoned => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        for original in released {
            self.publish(MaterialRecord::Abandon { original }, request)?;
        }
        let released_pins = {
            let state = self.state();
            state
                .catalog
                .pins
                .iter()
                .filter_map(|(identity, row)| match row.owner {
                    worth_foundational::facade::CorrectionMaterialPinOwner::Publication {
                        original,
                    } if !state.catalog.contains_original(original)
                        && originals.original_state(original).is_some_and(|state| {
                            state.fate() == worth_foundational::facade::WorldOriginalFate::Released
                        }) =>
                    {
                        Some((*identity, original))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        for (identity, original) in released_pins {
            self.publish(
                MaterialRecord::AbandonPublicationPin {
                    identity,
                    original,
                    class: worth_foundational::facade::RetentionClass::OptionalRetainedMaterial,
                },
                request,
            )?;
        }
        self.release_published_pins(originals, request)
    }
    fn release_published_pins(
        &self,
        originals: &worth_runtime_world::facade::WorldProgramActivationHistory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let published_pins = {
            let state = self.state();
            state
                .catalog
                .pins
                .iter()
                .filter_map(|(identity, row)| match row.owner {
                    worth_foundational::facade::CorrectionMaterialPinOwner::Publication {
                        original,
                    } if originals.original_state(original).is_some_and(|orig| {
                        matches!(
                            orig.fate(),
                            worth_foundational::facade::WorldOriginalFate::Published
                                | worth_foundational::facade::WorldOriginalFate::Continued
                        ) && originals
                            .original_observations()
                            .any(|obs| obs.identity() == orig.reserved_commit())
                    }) =>
                    {
                        Some((*identity, original))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        for (identity, original) in published_pins {
            self.publish(
                MaterialRecord::Release {
                    identity,
                    owner: worth_foundational::facade::CorrectionMaterialPinOwner::Publication {
                        original,
                    },
                    class: worth_foundational::facade::RetentionClass::OptionalRetainedMaterial,
                },
                request,
            )?;
        }
        Ok(())
    }
    pub fn ensure_capture(
        &self,
        material: MaterialCapture,
        world_commit_ordinal: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.settle(request)?;
        let original = material.original;
        {
            let mut state = self.state();
            state.world_ordinals.insert(original, world_commit_ordinal);
            let key = (material.original, material.member);
            if let Some(existing) = state.catalog.material.get(&key) {
                let capture = match &existing.body {
                    MaterialBody::Captured(capture) | MaterialBody::Ready { capture, .. } => {
                        capture
                    }
                    MaterialBody::Expired { .. } | MaterialBody::Abandoned => {
                        return Err(Denial::Unavailable)
                    }
                };
                return if **capture == material {
                    Ok(())
                } else {
                    Err(Denial::OriginalMismatch)
                };
            }
        }
        self.publish(
            MaterialRecord::Capture {
                material: Box::new(material),
            },
            request,
        )
    }
    pub fn ensure_ready(
        &self,
        material: MaterialReady,
        world_commit_ordinal: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.settle(request)?;
        let original = material.original;
        {
            let mut state = self.state();
            state.world_ordinals.insert(original, world_commit_ordinal);
            let existing = state
                .catalog
                .material
                .get(&(material.original, material.member))
                .ok_or(Denial::Unavailable)?;
            match &existing.body {
                MaterialBody::Ready { ready, .. } => {
                    return if **ready == material {
                        Ok(())
                    } else {
                        Err(Denial::OriginalMismatch)
                    }
                }
                MaterialBody::Captured(_) => {}
                MaterialBody::Expired { .. } | MaterialBody::Abandoned => {
                    return Err(Denial::Unavailable)
                }
            }
        }
        self.publish(
            MaterialRecord::Ready {
                material: Box::new(material),
            },
            request,
        )
    }
}
