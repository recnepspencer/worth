use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, RuntimeWorldBranchAdmissionDenial,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationToken, RuntimeWorldServiceDenial,
};

mod completion;
mod release;
use super::{activation::WorthQueryProductActivationDenial, WorthQueryProductRuntime};
use crate::basis::WorthQueryProductBranchLease;
pub use release::{
    WorthQueryProductBranchCreationNoEffect, WorthQueryProductBranchCreationRelease,
};

#[derive(Debug)]
pub enum WorthQueryProductBranchCreationOutcome {
    NoEffect(WorthQueryProductBranchCreationNoEffect),
    ReleasedOriginal(WorthQueryProductBranchCreationRelease),
    Performed(worth_runtime_world::facade::ProductBranchObservation),
    ProductUnpublished(crate::basis::WorthQueryProductBranchCreationRecovery),
}

#[derive(Debug)]
pub enum WorthQueryProductBranchCreationDenial {
    CoordinationCapacityExhausted,
    CoordinationUnavailable,
    ProgramSupportUnavailable,
    World(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
}

impl WorthQueryProductRuntime {
    /// World owns creation and the returned occurrence. Query reserves only
    /// bounded activation coordination before any component owner can move.
    pub(crate) fn create_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryProductBranchLease,
        source_program: Option<&ApplicationProgramRevision>,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        mut output_lineage: Option<std::sync::Arc<std::sync::Mutex<crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage>>>,
    ) -> Result<WorthQueryProductBranchCreationOutcome, WorthQueryProductBranchCreationDenial> {
        if source.observation().owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchCreationDenial::World(
                RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchAdmissionDenial::ForeignOwner),
            ));
        }
        let mut reservation = Some(
            self.activations
                .reserve_for_source_program(source_program)
                .map_err(|denial| match denial {
                    WorthQueryProductActivationDenial::CapacityExhausted => {
                        WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted
                    }
                    WorthQueryProductActivationDenial::ProgramSupportUnavailable => {
                        WorthQueryProductBranchCreationDenial::ProgramSupportUnavailable
                    }
                    _ => WorthQueryProductBranchCreationDenial::CoordinationUnavailable,
                })?,
        );
        let outcome = match &self.branch_program_source_owner {
            Some(owner) => {
                let revision = source_program
                    .ok_or(WorthQueryProductBranchCreationDenial::ProgramSupportUnavailable)?;
                let revision = *revision.as_bytes();
                let commitment =
                    worth_runtime_world::facade::world_branch_program_source_commitment(
                        source.observation(),
                        revision,
                    )
                    .map_err(|_| {
                        WorthQueryProductBranchCreationDenial::ProgramSupportUnavailable
                    })?;
                let request =
                    worth_runtime_world::facade::RuntimeWorldHomeBranchCreationRequest::new(
                        source.observation().clone(),
                        owner.admit_source(revision, commitment),
                        intent,
                    );
                let completion = completion::QueryBranchCreationCompletion::new(
                    execution,
                    reservation.take().expect("original coordination"),
                    output_lineage.take(),
                )?;
                self.owner.branch_port().create_home_product_branch(
                    execution,
                    request.with_completion(Box::new(completion)),
                    cancellation,
                )
            }
            None => self.owner.branch_port().create_product_branch(
                execution,
                source.observation().clone(),
                intent,
                cancellation,
            ),
        }
        .map_err(WorthQueryProductBranchCreationDenial::World)?;
        Ok(match outcome {
            RuntimeWorldBranchCreationOutcome::NoEffect(actual) => {
                WorthQueryProductBranchCreationOutcome::NoEffect(
                    WorthQueryProductBranchCreationNoEffect::from_world(actual),
                )
            }
            RuntimeWorldBranchCreationOutcome::ReleasedOriginal(actual) => {
                WorthQueryProductBranchCreationOutcome::ReleasedOriginal(
                    WorthQueryProductBranchCreationRelease::from_world(actual),
                )
            }
            RuntimeWorldBranchCreationOutcome::Performed(observation) => {
                if let Some(reservation) = reservation.take() {
                    reservation.commit(&observation);
                    if let Some(lineage) = output_lineage.take() {
                        lineage
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .register_fork(source.observation(), &observation);
                    }
                }
                WorthQueryProductBranchCreationOutcome::Performed(observation)
            }
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                let recovery = crate::basis::WorthQueryProductBranchCreationRecovery::new(
                    effects,
                    self.owner.recovery_port(),
                    self.clone(),
                );
                WorthQueryProductBranchCreationOutcome::ProductUnpublished(recovery)
            }
        })
    }
}

mod court_doors_for_test;
