use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramAdoptionRequestWithRequirements,
};

pub struct WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema> {
    pub(super) application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    // Principal-specific adoption authorization enters with Batch 3 custody.
    // Retain the authenticated request binding now so that surface does not drift.
    pub(super) _principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
}

pub enum WorthQueryApplicationProgramAdoptionRecoveryFailure {
    ProductSelection {
        denial:
            worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
        recovery: worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecovery,
    },
    Recovery(worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionRecoveryFailure),
}

#[derive(Debug)]
pub enum WorthQueryApplicationProgramInspectionDenial {
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

pub struct WorthQueryApplicationProgramAdoptionRequest<
    'application,
    'principal,
    'scope,
    'target,
    Schema,
> {
    programs: WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>,
    target: &'target ApplicationProgramRevision,
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
        self.application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?
            .branch_adoption_requirements(target)
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)
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
        self.application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationProgramInspectionDenial::ProductSelection)?
            .inspect_selected_program()
            .map_err(WorthQueryApplicationProgramInspectionDenial::Inspection)
    }

    pub fn adopt<'target>(
        self,
        target: &'target ApplicationProgramRevision,
    ) -> WorthQueryApplicationProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'target,
        Schema,
    > {
        WorthQueryApplicationProgramAdoptionRequest {
            programs: self,
            target,
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

impl<'application, 'principal, 'scope, 'target, Schema>
    WorthQueryApplicationProgramAdoptionRequest<'application, 'principal, 'scope, 'target, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn requirements<'requirements>(
        self,
        requirements: &'requirements WorthQueryProgramAdoptionRequirements,
    ) -> WorthQueryApplicationProgramAdoptionRequestWithRequirements<
        'application,
        'principal,
        'scope,
        'target,
        'requirements,
        Schema,
    > {
        WorthQueryApplicationProgramAdoptionRequestWithRequirements {
            programs: self.programs,
            target: self.target,
            requirements,
        }
    }
}
