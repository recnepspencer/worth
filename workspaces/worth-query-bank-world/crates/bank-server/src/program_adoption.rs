use bank_domain::schema::BankSchema;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramInspectionDenial, WorthQueryPreparedBranchAdoption,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::primary_graph::WorthQuerySelectedProgramInspection;
use worth_query_host::facade::product::WorthQueryProductBranch;

use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

#[derive(Debug)]
pub enum BankProgramInspectionDenial {
    Query(WorthQueryApplicationProgramInspectionDenial),
}

#[derive(Debug)]
pub enum BankProgramAdoptionPreparationDenial {
    TargetUnsupported,
    Query(Box<WorthQueryApplicationProgramAdoptionPreparationDenial>),
}

impl BankIdentityRuntime {
    /// Inspects the program actually carried by one Bank branch through the
    /// ordinary authenticated Query entry.
    pub fn inspect_branch_program(
        &self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
    ) -> Result<WorthQuerySelectedProgramInspection, BankProgramInspectionDenial> {
        self.request(principal, scope)
            .on_branch(branch)
            .programs()
            .inspect()
            .map_err(BankProgramInspectionDenial::Query)
    }

    /// Prepares adoption of one program explicitly rostered by this Bank host.
    /// Publication remains available only on the returned move-only product.
    pub fn prepare_branch_program_adoption<Target>(
        &self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        maximum_selection_work: usize,
    ) -> Result<WorthQueryPreparedBranchAdoption, BankProgramAdoptionPreparationDenial>
    where
        Target: ApplicationProgramDefinition<BankSchema> + 'static,
    {
        let target = self
            .application_program()
            .supported_program::<Target>()
            .ok_or(BankProgramAdoptionPreparationDenial::TargetUnsupported)?
            .owned_revision()
            .clone();
        let programs = self.request(principal, scope).on_branch(branch).programs();
        let requirements = programs
            .compare(&target)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))?;
        programs
            .adopt(&target)
            .requirements(&requirements)
            .prepare(maximum_selection_work)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))
    }
}
