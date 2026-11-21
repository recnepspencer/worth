//! Installed original operation custody on Query's existing obligation history.
use super::{query_obligations::*, WorthQueryAdvancementDenial, WorthQueryAdvancementPhase};
use worth_foundational::facade::ProgramSupportObligationIdentity;
use worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort;
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
    pub fn from_parts(
        _seal: crate::ports::WorthQueryQueryCrateSeal,
        identity: ProgramSupportObligationIdentity,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self { identity, owner }
    }

    pub fn identity(&self) -> ProgramSupportObligationIdentity {
        self.identity
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
        _: &dyn WorthQueryProgramPublicationAccessPort,
    ) -> bool {
        self.runtime.product_runtime.is_directory_home()
    }

    pub fn admit_original_operation_retention(
        &self,
        owner: Option<&WorthQueryOriginalOperationOwner>,
        access: &dyn WorthQueryProgramPublicationAccessPort,
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
        _: &dyn WorthQueryProgramPublicationAccessPort,
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
        access: &dyn WorthQueryProgramPublicationAccessPort,
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
