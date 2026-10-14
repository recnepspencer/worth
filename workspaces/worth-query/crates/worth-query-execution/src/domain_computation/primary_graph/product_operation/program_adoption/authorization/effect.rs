//! Erases the native operation requested, while typed dispatch keeps authority.
use super::super::preparation::{
    WorthQueryBranchAdoptionPreparationDenial, WorthQueryPreparedBranchAdoption,
    WorthQueryPreparedProgramMigration,
};
use super::super::{preparation, publication};
use super::super::{
    WorthQueryBranchAdoptionRecovery, WorthQueryBranchAdoptionRecoveryFailure,
    WorthQueryBranchAdoptionRecoveryOutcome,
};
use super::*;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryWorkflowDispositions;
use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
pub(in crate::domain_computation::primary_graph) enum AdoptionEffect<'a> {
    Read,
    Prepare {
        requirements: &'a WorthQueryProgramAdoptionRequirements,
        migration: Option<WorthQueryPreparedProgramMigration>,
        workflow: Option<WorthQueryWorkflowDispositions>,
        maximum_selection_work: usize,
    },
    Publish(WorthQueryPreparedBranchAdoption),
    Recover(WorthQueryBranchAdoptionRecovery),
}
pub(in crate::domain_computation::primary_graph) enum AdoptionEffectOutcome {
    Read(
        Result<
            (),
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
        >,
    ),
    Prepared(Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial>),
    Published(publication::WorthQueryBranchAdoptionPublicationOutcome),
    Recovered(
        Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>,
    ),
}
pub(in crate::domain_computation::primary_graph) struct AdoptionEffectRefusal<'a> {
    pub cause: WorthQueryProgramAdoptionAuthorizationDenial,
    pub effect: AdoptionEffect<'a>,
}
impl AdoptionEffect<'_> {
    pub(super) fn consume<Schema: ApplicationSchema, Operation, Input, Scope>(
        self,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        target: &ApplicationProgramRevision,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> AdoptionEffectOutcome {
        let execution = phase
            .execution_request_for(&selected.application().product_runtime)
            .expect("capability progression checked the phase owner");
        match self {
            Self::Read => AdoptionEffectOutcome::Read(admission.validate_current_authority()),
            Self::Prepare {
                requirements,
                migration,
                workflow,
                maximum_selection_work,
            } => {
                let prepared = (|| {
                    let gate = selected
                        .application()
                        .product_runtime
                        .activations
                        .gate(selected.product().branch_identity())
                        .map_err(|denial| {
                            WorthQueryBranchAdoptionPreparationDenial::ProductActivation(
                                denial.into(),
                            )
                        })?;
                    gate.publish(|| {
                        preparation::prepare(
                            &execution,
                            admission,
                            phase,
                            selected,
                            target,
                            requirements,
                            migration,
                            workflow,
                            maximum_selection_work,
                            request,
                        )
                    })
                    .map_err(|denial| {
                        WorthQueryBranchAdoptionPreparationDenial::ProductActivation(denial.into())
                    })?
                })();
                AdoptionEffectOutcome::Prepared(prepared)
            }
            Self::Publish(prepared) => {
                AdoptionEffectOutcome::Published(prepared.publish_in_advancement(phase, admission))
            }
            Self::Recover(recovery) => {
                AdoptionEffectOutcome::Recovered(selected.recover_branch_adoption_in_advancement(
                    &execution, admission, recovery, request,
                ))
            }
        }
    }
}
