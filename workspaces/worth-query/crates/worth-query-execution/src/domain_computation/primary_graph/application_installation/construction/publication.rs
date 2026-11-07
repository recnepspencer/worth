//! Configuration, publication and completion consume returning owned phases.
use super::native_preparation::PreparedMemoryApplication;
use super::*;
use crate::domain_computation::primary_graph::bootstrap::{
    BootstrapPublicationReservation, BootstrapPublicationStorage,
};
mod custody;
use custody::*;
mod completion;
mod configuration;
mod service;
#[inline(never)]
pub(super) fn publish<S: ApplicationSchemaComposition>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    prepared: BootstrapPublicationStorage<PreparedMemoryApplication<S>>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<S>, WorthQueryApplicationPreparationDenial> {
    let configured = configuration::prepare(phase, prepared)?;
    let published = service::publish(phase, configured)?;
    completion::complete(phase, published)
}
