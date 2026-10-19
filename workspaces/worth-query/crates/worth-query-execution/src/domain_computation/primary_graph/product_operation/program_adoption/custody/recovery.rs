use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
mod readmission;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{NoEffectCompositePublication, RuntimeWorldRecoveryDenial};

use super::super::publication::{
    WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

mod release;
pub use release::WorthQueryBranchAdoptionRecoveryReleaseFailure;

const IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS: u64 = 0;

/// Custody of a branch adoption whose publication moved some owners but not the
/// product head.
///
/// Get one from an unpublished adoption's `into_recovery`. Continue it with
/// `recover_branch_adoption`, or release its custody with
/// `release_branch_adoption_recovery`. It holds the exact custody needed to
/// finish.
#[must_use = "unpublished adoption recovery owns exact World custody"]
pub struct WorthQueryBranchAdoptionRecovery {
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    selected_entity_count: usize,
    migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
    custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
    product: WorthQueryProductUnpublishedRecovery,
    support_custody:
        crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
    // Last to drop: physical backing releases before its retained grant.
    candidate_funding: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CandidateFunding>>,
}

impl WorthQueryBranchAdoptionRecovery {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn new(
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
        custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
        candidate_funding: Option<
            std::sync::Arc<
                crate::domain_computation::primary_graph::application_attempt::CandidateFunding,
            >,
        >,
        product: WorthQueryProductUnpublishedRecovery,
        support_custody: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
    ) -> Self {
        Self {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            candidate_funding,
            product,
            support_custody,
        }
    }

    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub const fn migration(
        &self,
    ) -> Option<&super::super::preparation::WorthQueryProgramMigrationDescription> {
        self.migration.as_ref()
    }

    pub const fn custody(
        &self,
    ) -> &super::super::preparation::WorthQueryProgramCustodyDispositionInventory {
        &self.custody
    }
}

/// Why continuing a branch adoption recovery stopped. No adoption was
/// published, and the recovery is handed back to continue again.
#[derive(Debug)]
pub enum WorthQueryBranchAdoptionRecoveryDenial {
    Authorization(super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial),
    /// The recovery call was refused before inspecting its owners.
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The recovery expects a different product than the one selected.
    ProductAffinityMismatch,
    /// Runtime World could not inspect the unpublished owner effects.
    Inspection(RuntimeWorldRecoveryDenial),
    /// Owner settlement of the unpublished effects did not finish.
    OwnerSettlement(RuntimeWorldRecoveryDenial),
    /// Adopting the settled owner effects into the product head could not be
    /// prepared.
    AdoptionPreparation(worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial),
}

/// A branch adoption recovery step that did not finish: the denial, and the
/// recovery handed back to continue.
pub struct WorthQueryBranchAdoptionRecoveryFailure {
    denial: WorthQueryBranchAdoptionRecoveryDenial,
    recovery: WorthQueryBranchAdoptionRecovery,
}

impl WorthQueryBranchAdoptionRecoveryFailure {
    pub const fn denial(&self) -> &WorthQueryBranchAdoptionRecoveryDenial {
        &self.denial
    }

    pub fn into_recovery(self) -> WorthQueryBranchAdoptionRecovery {
        self.recovery
    }
}

/// What continuing a branch adoption recovery did. Returned by
/// `recover_branch_adoption`.
pub enum WorthQueryBranchAdoptionRecoveryOutcome {
    /// The adoption is published. `cleanup` reports releasing the recovery's
    /// owner custody; a cleanup failure does not undo the adoption.
    Performed {
        adoption: WorthQueryPerformedBranchAdoption,
        cleanup: Result<
            crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
    /// The publication had no effect: nothing was published. The recovery is
    /// handed back to continue again.
    NoEffect {
        no_effect: NoEffectCompositePublication,
        recovery: WorthQueryBranchAdoptionRecovery,
    },
    /// The publication again moved some owners but not the product head. `next`
    /// is the new unpublished adoption; `prior_cleanup` reports releasing the
    /// earlier recovery's custody.
    ProductUnpublished {
        next: WorthQueryUnpublishedBranchAdoption,
        prior_cleanup: Result<
            crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn recover_branch_adoption_authorized_in_advancement(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryBranchAdoptionRecovery,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
        let _execution =
            match phase.checked_execution_request_for(&self.application().product_runtime) {
                Ok(execution) => execution,
                Err(cause) => {
                    return Err(failure(
                        WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(cause),
                        recovery,
                    ))
                }
            };
        use super::super::authorization::{AdoptionEffect, AdoptionEffectOutcome};
        let target = *recovery.target();
        match self.with_program_adoption(
            phase,
            principal,
            request,
            &target,
            AdoptionEffect::Recover(recovery),
        ) {
            Ok(AdoptionEffectOutcome::Recovered(outcome)) => outcome,
            Err(refusal) => {
                let AdoptionEffect::Recover(recovery) = refusal.effect else {
                    unreachable!("recovery refusal retains the requested native custody")
                };
                Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Authorization(refusal.cause),
                    recovery,
                ))
            }
            Ok(_) => unreachable!("recovery dispatch preserves the requested effect"),
        }
    }

    pub fn recover_branch_adoption(
        &self,
        recovery: WorthQueryBranchAdoptionRecovery,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
        let mut retained = Some(recovery);
        let result = self
            .application()
            .with_application_advancement(request, |phase| {
                self.recover_branch_adoption_authorized_in_advancement(
                    &phase,
                    retained.take().expect("one admitted recovery"),
                    principal,
                    request,
                )
            });
        match result {
            Ok(result) => result,
            Err(cause) => Err(WorthQueryBranchAdoptionRecoveryFailure {
                denial: WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(cause),
                recovery: retained.expect("refusal precedes admission"),
            }),
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn integration_recover_branch_adoption(
        &self,

        branch: crate::basis::WorthQueryProductBranch,
        recovery: WorthQueryBranchAdoptionRecovery,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>,
        (
            crate::basis::WorthQueryProductBranchAdmissionDenial,
            WorthQueryBranchAdoptionRecovery,
        ),
    > {
        let mut retained = Some(recovery);
        self.with_application_advancement(request, |phase| {
            let phase_request = match phase.execution_request_for(&self.product_runtime) {
                Ok(request) => request,
                Err(_) => {
                    return Err((
                        crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
                            AdmissionRefusal::ForeignPhase,
                        ),
                        retained.take().expect("one retained recovery"),
                    ))
                }
            };
            let execution = &phase_request;
            let recovery = retained.take().expect("one admitted recovery");

            let product = match self
                .product_runtime
                .admit_product_occurrence(branch.occurrence())
            {
                Ok(product) => product,
                Err(denial) => return Err((denial, recovery)),
            };
            let selected = match self.on_product_for_adoption_recovery(execution, product) {
                Ok(selected) => selected,
                Err(denial) => return Err((denial, recovery)),
            };
            Ok(selected.recover_branch_adoption_authorized_in_advancement(
                &phase, recovery, principal, request,
            ))
        })
        .unwrap_or_else(|cause| {
            Ok(Err(failure(
                WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(cause),
                retained.expect("a refused root keeps recovery facts"),
            )))
        })
    }
}

fn failure(
    denial: WorthQueryBranchAdoptionRecoveryDenial,
    recovery: WorthQueryBranchAdoptionRecovery,
) -> WorthQueryBranchAdoptionRecoveryFailure {
    WorthQueryBranchAdoptionRecoveryFailure { denial, recovery }
}
