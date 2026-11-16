//! Installed original operation custody on Query's existing obligation history.
use super::application_installation::WorthQueryProgramApplicationRuntime;
use super::{query_obligations::*, WorthQueryAdvancementDenial, WorthQueryAdvancementPhase};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_foundational::facade::ProgramSupportObligationIdentity;
use worth_query_installation::facade::ApplicationSchema;

#[derive(Debug)]
pub enum WorthQueryOriginalOperationDenial {
    OwnershipMaterialUnavailable,
    Principal(super::WorthQueryPrincipalResolutionDenial),
    Scope(super::WorthQueryEntityResolutionDenial),
    QueryAdmission(super::WorthQueryApplicationQueryAdmissionDenial),
    QueryExecution(super::WorthQueryApplicationOneShotDenial),
    StaleIncarnation,
    ForeignHome,
    Execution(WorthQueryAdvancementDenial),
    Obligation(QueryObligationDenial),
}
/// The live incarnation is process-local; only identity is used for cold lookup.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryOriginalOperationOwner {
    identity: ProgramSupportObligationIdentity,
    owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
}
impl WorthQueryOriginalOperationOwner {
    pub fn identity(&self) -> ProgramSupportObligationIdentity {
        self.identity
    }
}
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
        _: &WorthQueryProgramPublicationAccess,
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
            WorthQueryOriginalOperationOwner {
                identity,
                owner: self
                    .runtime()
                    .runtime
                    .product_runtime
                    .owner
                    .owner_identity(),
            },
            pending,
        ))
    }
    pub fn readmit_original_operation_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
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
            WorthQueryOriginalOperationOwner {
                identity,
                owner: self
                    .runtime()
                    .runtime
                    .product_runtime
                    .owner
                    .owner_identity(),
            },
            row,
        ))
    }
}
impl<S: ApplicationSchema> super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    fn original_history(
        &self,
    ) -> Result<
        &super::query_obligations::history::QueryObligationHistory,
        WorthQueryOriginalOperationDenial,
    > {
        self.installed_program_support()
            .map(|support| support.lifecycle())
            .ok_or(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable)
    }
    pub fn original_operation_requires_record(
        &self,
        _: &WorthQueryProgramPublicationAccess,
    ) -> bool {
        self.runtime.product_runtime.is_directory_home()
    }
    pub fn admit_original_operation_retention(
        &self,
        owner: Option<&WorthQueryOriginalOperationOwner>,
        access: &WorthQueryProgramPublicationAccess,
    ) -> Result<(), WorthQueryOriginalOperationDenial> {
        match owner {
            Some(owner) => self.admit_original_operation_owner(owner, access),
            None if self.original_operation_requires_record(access) => {
                Err(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable)
            }
            None => Ok(()),
        }
    }
    pub fn admit_original_operation_owner(
        &self,
        owner: &WorthQueryOriginalOperationOwner,
        _: &WorthQueryProgramPublicationAccess,
    ) -> Result<(), WorthQueryOriginalOperationDenial> {
        use WorthQueryOriginalOperationDenial as D;
        self.runtime
            .product_runtime
            .owner
            .owner_identity()
            .admit_incarnation(owner.owner)
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::StaleIncarnation => D::StaleIncarnation,
                worth_proof::HomeIncarnationDenial::ForeignHome => D::ForeignHome,
            })
    }
    pub fn progress_original_operation_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        owner: &WorthQueryOriginalOperationOwner,
        position: Option<QueryOriginalOperationPosition>,
    ) -> Result<(), WorthQueryOriginalOperationDenial> {
        use WorthQueryOriginalOperationDenial as D;
        self.admit_original_operation_owner(owner, access)?;
        let request = phase
            .checked_execution_request_for(self.product_runtime())
            .map_err(D::Execution)?;
        let record = match position {
            Some(position) => QueryOriginalOperationRecord::Advance {
                identity: owner.identity,
                position,
            },
            None => QueryOriginalOperationRecord::Discharge {
                identity: owner.identity,
            },
        };
        self.original_history()?
            .progress_original_operation(&request, record)
            .map_err(D::Obligation)
    }
}
