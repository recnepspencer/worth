//! Owning expiry keeps terminal metadata and serializes the joined disposal.
use super::catalog::MaterialBody;
use super::*;
use worth_foundational::facade::{
    CanonicalDigestId, DurableCompositePublicationAttemptIdentity as Original,
};
impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn settle_retention(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<MaterialPublication<'_>, Denial> {
        let driver = self.publication()?;
        driver.finish_pending(self.medium()?.as_ref(), request)?;
        Ok(driver)
    }
    pub(in crate::domain_computation::primary_graph) fn expire_at_coordinates(
        &self,
        coordinates: &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
        originals: &worth_runtime_world::facade::WorldProgramActivationHistory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<MaterialPublication<'_>, Denial> {
        self.admit_coordinate_home(coordinates)?;
        let driver = self.settle_retention(request)?;
        let original = self.original_at_coordinates(coordinates, originals)?;
        let expired = {
            let state = self.state();
            let entry = state
                .catalog
                .material
                .get(&original)
                .ok_or(Denial::Unavailable)?;
            match &entry.body {
                MaterialBody::Expired { .. } => true,
                MaterialBody::Ready { .. } => false,
                MaterialBody::Captured(_) | MaterialBody::Abandoned => {
                    return Err(Denial::MaterialNotReady)
                }
            }
        };
        if !expired {
            driver.publish(MaterialRecord::Expire { original }, request)?;
        }
        Ok(driver)
    }
    fn admit_coordinate_home(
        &self,
        coordinates: &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
    ) -> Result<(), Denial> {
        let medium = self.medium()?;
        let worth_foundational::facade::WorldOwnerCoordinate::Home { owner } =
            coordinates.world_owner()
        else {
            return Err(Denial::ForeignHome);
        };
        if owner.home() != medium.home() {
            return Err(Denial::ForeignHome);
        }
        Ok(())
    }
    fn original_at_coordinates(
        &self,
        coordinates: &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
        originals: &worth_runtime_world::facade::WorldProgramActivationHistory,
    ) -> Result<Original, Denial> {
        let worth_foundational::facade::WorldOwnerCoordinate::Home { owner } =
            coordinates.world_owner()
        else {
            return Err(Denial::ForeignHome);
        };
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut matches = state
            .catalog
            .material
            .iter()
            .filter(|(_, entry)| match &entry.body {
                MaterialBody::Ready { ready, .. } => &ready.commit == coordinates.relational(),
                MaterialBody::Expired { commit } => commit == coordinates.relational(),
                MaterialBody::Captured(_) | MaterialBody::Abandoned => false,
            });
        let (identity, entry) = matches.next().ok_or(Denial::Unavailable)?;
        if matches.next().is_some() {
            return Err(Denial::OriginalMismatch);
        }
        let original = originals
            .original_state(*identity)
            .ok_or(Denial::OriginalMismatch)?;
        if original.reserved_commit().owner() != owner
            || original.reserved_commit().ordinal() != coordinates.world_commit()
            || original.program_revision().bytes() != &entry.metadata.program
        {
            return Err(Denial::OriginalMismatch);
        }
        match original.fate() {
            worth_foundational::facade::WorldOriginalFate::Published => {}
            worth_foundational::facade::WorldOriginalFate::Open
            | worth_foundational::facade::WorldOriginalFate::Released
            | worth_foundational::facade::WorldOriginalFate::Continued => {
                return Err(Denial::OriginalMismatch)
            }
        }
        Ok(*identity)
    }
}
impl MaterialPublication<'_> {
    pub(in crate::domain_computation::primary_graph) fn admit_obligation_scope(
        &self,
        medium: &dyn crate::facade::query_obligations::QueryObligationsMedium,
    ) -> Result<(), Denial> {
        let actual = self.owner.medium()?;
        if actual.home() != medium.home() {
            return Err(Denial::ForeignHome);
        }
        if actual.incarnation() != medium.incarnation() {
            return Err(Denial::StaleIncarnation);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn expired_membership(
        &self,
        original: Original,
        revision: CanonicalDigestId,
    ) -> bool {
        let state = self.owner.state();
        state.pending.is_none()
            && state.catalog.material.get(&original).is_some_and(|entry| {
                entry.metadata.program == *revision.bytes()
                    && matches!(&entry.body, MaterialBody::Expired { .. })
            })
    }
    pub(in crate::domain_computation::primary_graph) fn has_expired_material(&self) -> bool {
        self.owner
            .state()
            .catalog
            .material
            .values()
            .any(|entry| matches!(&entry.body, MaterialBody::Expired { .. }))
    }
}
