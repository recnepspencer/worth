//! The Query receiver owns semantic material, publication and mandatory uses.
pub(super) mod catalog;
mod expiry;
mod journal;
mod mutation;
mod observation;
mod pins;
mod prepared_commands;
mod readmission;
mod recovery_pins;
mod retention_denial;
pub use retention_denial::WorthQueryCorrectionRetentionDenial;
pub(super) mod record;
use crate::domain_computation::application_aftermath::correction_material::{
    QueryCorrectionMaterialDenial as Denial, QueryCorrectionMaterialFrame, QueryCorrectionsMedium,
};
use catalog::MaterialCatalog;
pub(in crate::domain_computation) use observation::RetainedCorrectionMaterial;
use record::MaterialRecord;
use std::sync::{Arc, Mutex, MutexGuard};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct CorrectionMaterialOwner {
    medium: Mutex<Option<Arc<dyn QueryCorrectionsMedium>>>,
    publication: Mutex<()>,
    state: Mutex<MaterialState>,
}
pub(in crate::domain_computation::primary_graph) struct MaterialPublication<'a> {
    owner: &'a CorrectionMaterialOwner,
    _driver: MutexGuard<'a, ()>,
}
#[derive(Debug)]
struct MaterialState {
    catalog: MaterialCatalog,
    allocated_generation: u64,
    pending: Option<Arc<QueryCorrectionMaterialFrame>>,
    pins: pins::LiveMaterialPins,
    funding: Arc<MaterialFunding>,
}
#[derive(Debug)]
struct MaterialFunding(Mutex<Vec<Option<worth_execution::ExecutionMemoryReservation>>>);
impl MaterialFunding {
    fn retain(&self, funding: Option<worth_execution::ExecutionMemoryReservation>) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(funding);
    }
}
impl CorrectionMaterialOwner {
    fn publish(
        &self,
        record: MaterialRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.publication()?.publish(record, request)
    }
    fn settle(&self, request: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
        self.publication()?
            .finish_pending(self.medium()?.as_ref(), request)
    }
    pub(in crate::domain_computation::primary_graph) fn admit_medium(
        &self,
        presented: &dyn QueryCorrectionsMedium,
    ) -> Result<(), Denial> {
        self.medium()?
            .authority()
            .admit(presented.authority())
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignHome,
                worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            })
    }
    fn state(&self) -> MutexGuard<'_, MaterialState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn medium(&self) -> Result<Arc<dyn QueryCorrectionsMedium>, Denial> {
        self.medium
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .cloned()
            .ok_or(Denial::OwnerUnavailable)
    }
    fn publication(&self) -> Result<MaterialPublication<'_>, Denial> {
        match self.publication.try_lock() {
            Ok(guard) => Ok(MaterialPublication {
                owner: self,
                _driver: guard,
            }),
            Err(std::sync::TryLockError::Poisoned(cause)) => Ok(MaterialPublication {
                owner: self,
                _driver: cause.into_inner(),
            }),
            Err(std::sync::TryLockError::WouldBlock) => Err(Denial::PublicationPending),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn admission(
        &self,
    ) -> Result<worth_proof::AdmittedHomeQueryCorrections, Denial> {
        let medium = self.medium()?;
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        Ok(medium.authority().admit_query_correction_prefix(
            state.catalog.position,
            state.catalog.digest,
            state.catalog.payload_digest,
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn discharge_unperformed(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let driver = self.publication()?;
        let medium = self.medium()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let pins = {
            let state = self.state();
            state
                .catalog
                .pins
                .iter()
                .filter(|(identity, _)| state.pins.contains(**identity))
                .map(|(identity, row)| (*identity, row.owner))
                .collect::<Vec<_>>()
        };
        for (identity, owner) in pins {
            match owner {
                worth_foundational::facade::CorrectionMaterialPinOwner::Recovery { .. } => continue,
                worth_foundational::facade::CorrectionMaterialPinOwner::PreparedCommand {
                    ..
                } => {}
                worth_foundational::facade::CorrectionMaterialPinOwner::Publication { .. } => {
                    return Err(Denial::PublicationPending)
                }
            }
            let worth_foundational::facade::CorrectionMaterialPinOwner::PreparedCommand { key } =
                owner
            else {
                unreachable!("publication custody was refused above");
            };
            driver.publish(
                MaterialRecord::DischargePreparation {
                    identity,
                    key,
                    class: worth_foundational::facade::RetentionClass::OptionalRetainedMaterial,
                },
                request,
            )?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn finish_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.discharge_unperformed(request)?;
        self.medium
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        Ok(())
    }
}
