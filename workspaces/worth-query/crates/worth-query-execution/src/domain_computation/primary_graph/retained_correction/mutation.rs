//! Idempotent semantic publication by the original acknowledged owner.
use super::catalog::MaterialBody;
use super::record::{MaterialCapture, MaterialReady};
use super::*;
impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn abandon_released(
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
            .filter_map(|(identity, material)| match &material.body {
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
            .collect::<Vec<_>>();
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
                    } if !state.catalog.material.contains_key(&original)
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
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn ensure_capture(
        &self,
        material: MaterialCapture,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.settle(request)?;
        {
            let state = self.state();
            if let Some(existing) = state.catalog.material.get(&material.original) {
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
    pub(in crate::domain_computation::primary_graph) fn ensure_ready(
        &self,
        material: MaterialReady,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.settle(request)?;
        {
            let state = self.state();
            let existing = state
                .catalog
                .material
                .get(&material.original)
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
