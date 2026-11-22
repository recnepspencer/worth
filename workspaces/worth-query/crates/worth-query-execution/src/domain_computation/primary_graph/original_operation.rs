//! Installed original operation custody on Query's existing obligation history.
use super::application_installation::WorthQueryProgramApplicationRuntime;
use super::{query_obligations::*, WorthQueryAdvancementPhase};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_foundational::facade::ProgramSupportObligationIdentity;
use worth_query_installation::facade::ApplicationSchema;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::original_operation::{
    WorthQueryOriginalOperationDenial, WorthQueryOriginalOperationOwner,
};

impl<S: ApplicationSchema, P> WorthQueryProgramApplicationRuntime<S, P> {
    fn original_history(
        &self,
    ) -> Result<
        &super::query_obligations::history::QueryObligationHistory,
        WorthQueryOriginalOperationDenial,
    > {
        self.runtime()
            .installed_program_support()
            .map(|support| support.lifecycle())
            .ok_or(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable)
    }

    pub fn retain_original_operation_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        data: QueryOriginalOperationData,
    ) -> Result<
        (
            WorthQueryOriginalOperationOwner,
            Option<QueryObligationDenial>,
        ),
        WorthQueryOriginalOperationDenial,
    > {
        use WorthQueryOriginalOperationDenial as D;
        if data.revision != *self.installed_program().revision().as_bytes()
            || data.program != self.installed_program().identity().as_str()
        {
            return Err(D::OwnershipMaterialUnavailable);
        }
        let request = phase
            .checked_execution_request_for(self.runtime().product_runtime())
            .map_err(D::Execution)?;
        let (identity, pending) = self
            .original_history()?
            .retain_original_operation(&request, data)
            .map_err(D::Obligation)?;
        Ok((
            WorthQueryOriginalOperationOwner::from_retained_obligation(
                access,
                &self.runtime().inner,
                identity,
            ),
            pending,
        ))
    }

    pub fn readmit_original_operation_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        identity: ProgramSupportObligationIdentity,
    ) -> Result<
        (
            WorthQueryOriginalOperationOwner,
            QueryOriginalOperationObligation,
        ),
        WorthQueryOriginalOperationDenial,
    > {
        use WorthQueryOriginalOperationDenial as D;
        let request = phase
            .checked_execution_request_for(self.runtime().product_runtime())
            .map_err(D::Execution)?;
        let row = self
            .original_history()?
            .original_operation(&request, identity)
            .map_err(|cause| match cause {
                QueryObligationDenial::UnknownObligation => D::OwnershipMaterialUnavailable,
                QueryObligationDenial::ForeignHome => D::ForeignHome,
                cause => D::Obligation(cause),
            })?;
        if row.data.revision != *self.installed_program().revision().as_bytes()
            || row.data.program != self.installed_program().identity().as_str()
        {
            return Err(D::OwnershipMaterialUnavailable);
        }
        Ok((
            WorthQueryOriginalOperationOwner::from_retained_obligation(
                access,
                &self.runtime().inner,
                identity,
            ),
            row,
        ))
    }
}
