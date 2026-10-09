use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramAdoptionRequest,
};

/// Program operations on one branch: compare, inspect, adopt and recover.
pub struct WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema> {
    pub(super) application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    // Principal-specific adoption authorization enters with Batch 3 custody.
    // Retain the authenticated request binding now so that surface does not drift.
    pub(super) _principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
}

/// Why recovering an interrupted program adoption failed. `into_recovery` returns the
/// recovery so it can be tried again.
pub enum WorthQueryApplicationProgramAdoptionRecoveryFailure {
    ProductSelection {
        denial:
            worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
        recovery: worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecovery,
    },
    Recovery(worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecoveryFailure),
}

/// Why the branch's adopted program could not be inspected.
#[derive(Debug)]
pub enum WorthQueryApplicationProgramInspectionDenial {
    /// The host call could not admit its execution request before reading.
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Inspection(
        worth_query_execution::facade::primary_graph::WorthQuerySelectedProgramInspectionDenial,
    ),
}

impl WorthQueryApplicationProgramAdoptionRecoveryFailure {
    pub fn into_recovery(
        self,
    ) -> worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecovery {
        match self {
            Self::ProductSelection { recovery, .. } => recovery,
            Self::Recovery(failure) => failure.into_recovery(),
        }
    }
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::application_entry) fn new(
        application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Self {
        Self {
            application,
            _principal: principal,
            scope,
            branch,
        }
    }

    pub fn compare(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<
        WorthQueryProgramAdoptionRequirements,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    > {
        let request_scope = self.scope.clone();
        let runtime = self.application;
        runtime.with_application_advancement(&request_scope, |_phase| {

        self.application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?
            .branch_adoption_requirements(target)
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)

        }).map_err(|cause| WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied(cause)))?
    }

    /// Reports the rostered program carried by this exact selected branch
    /// occurrence. The report is descriptive and cannot be used as adoption
    /// preparation or publication authority.
    pub fn inspect(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQuerySelectedProgramInspection,
        WorthQueryApplicationProgramInspectionDenial,
    > {
        let request_scope = self.scope.clone();
        let runtime = self.application;
        runtime
            .with_application_advancement(&request_scope, |_phase| {
                self.application
                    .on_branch(self.branch)
                    .select()
                    .map_err(WorthQueryApplicationProgramInspectionDenial::ProductSelection)?
                    .inspect_selected_program()
                    .map_err(WorthQueryApplicationProgramInspectionDenial::Inspection)
            })
            .map_err(|cause| {
                WorthQueryApplicationProgramInspectionDenial::ExecutionRequest(cause)
            })?
    }

    /// Adopts the target that `requirements`, as [`Self::compare`] returned
    /// them, name. Preparation refuses requirements this branch no longer
    /// matches.
    pub fn adopt<'requirements>(
        self,
        requirements: &'requirements WorthQueryProgramAdoptionRequirements,
    ) -> WorthQueryApplicationProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'requirements,
        Schema,
    > {
        WorthQueryApplicationProgramAdoptionRequest {
            programs: self,
            requirements,
            migration: None,
            workflow: None,
        }
    }

    /// Continues only domain custody returned by an unpublished adoption.
    /// A descriptive World handle cannot reach this effectful entry.
    ///
    /// ```compile_fail,E0308
    /// use worth_query_installation::facade::ApplicationSchema;
    /// use worth_query_publication::facade::application_entry::WorthQueryApplicationProgramsRequest;
    /// use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;
    ///
    /// fn raw_handle_cannot_resume<Schema: ApplicationSchema>(
    ///     programs: WorthQueryApplicationProgramsRequest<'_, '_, '_, Schema>,
    ///     raw: ProductUnpublishedRecoveryHandle,
    /// ) {
    ///     let _ = programs.recover(raw);
    /// }
    /// ```
    ///
    /// A performed World receipt is descriptive evidence, not Query's exact
    /// unpublished adoption custody.
    ///
    /// ```compile_fail,E0308
    /// use worth_query_installation::facade::ApplicationSchema;
    /// use worth_query_publication::facade::application_entry::WorthQueryApplicationProgramsRequest;
    /// use worth_runtime_world::facade::PerformedCompositePublication;
    ///
    /// fn raw_receipt_cannot_resume<Schema: ApplicationSchema>(
    ///     programs: WorthQueryApplicationProgramsRequest<'_, '_, '_, Schema>,
    ///     receipt: PerformedCompositePublication,
    /// ) {
    ///     let _ = programs.recover(receipt);
    /// }
    /// ```
    pub fn recover(
        self,
        recovery: worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecovery,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecoveryOutcome,
        WorthQueryApplicationProgramAdoptionRecoveryFailure,
    > {
        match self.application.integration_recover_branch_adoption(
            self.branch,
            recovery,
            self.scope,
        ) {
            Ok(outcome) => {
                outcome.map_err(WorthQueryApplicationProgramAdoptionRecoveryFailure::Recovery)
            }
            Err((denial, recovery)) => Err(
                WorthQueryApplicationProgramAdoptionRecoveryFailure::ProductSelection {
                    denial,
                    recovery,
                },
            ),
        }
    }
}
