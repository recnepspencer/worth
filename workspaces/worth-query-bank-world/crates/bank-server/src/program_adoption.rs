use bank_domain::schema::BankSchema;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramInspectionDenial, WorthQueryPreparedBranchAdoption,
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowDispositionDenial,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramDefinition, ApplicationProgramRevision,
};
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
    WorkflowDefinitionMismatch,
    LiveWorkflowInstances,
    WorkflowDisposition(WorthQueryWorkflowDispositionDenial),
    Query(Box<WorthQueryApplicationProgramAdoptionPreparationDenial>),
}

impl BankIdentityRuntime {
    /// The production branch this Bank host serves.
    pub fn current_branch(&self) -> WorthQueryProductBranch {
        self.application_program().current_world()
    }

    /// The revision of the program this Bank host installed.
    pub fn installed_program_revision(&self) -> &ApplicationProgramRevision {
        self.application_program().owned_revision()
    }

    /// The revision of one program this Bank host rosters beside its
    /// installed program, or `None` when the host does not roster it.
    pub fn supported_program_revision<Target>(&self) -> Option<ApplicationProgramRevision>
    where
        Target: ApplicationProgramDefinition<BankSchema> + 'static,
    {
        self.application_program()
            .supported_program::<Target>()
            .map(|owner| *owner.owned_revision())
    }

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
            .supported_program_revision::<Target>()
            .ok_or(BankProgramAdoptionPreparationDenial::TargetUnsupported)?;
        let programs = self.request(principal, scope).on_branch(branch).programs();
        let requirements = programs
            .compare(&target)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))?;
        programs
            .adopt(&requirements)
            .prepare(maximum_selection_work)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))
    }

    /// Retire one exact current definition while adopting a rostered program.
    /// A live instance must finish under its source program before this move.
    pub fn prepare_branch_program_adoption_retiring_definition<Target>(
        &self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        expected: &PublishedWorkflowDefinitionRef,
        maximum_work_units: usize,
    ) -> Result<WorthQueryPreparedBranchAdoption, BankProgramAdoptionPreparationDenial>
    where
        Target: ApplicationProgramDefinition<BankSchema> + 'static,
    {
        let target = self
            .supported_program_revision::<Target>()
            .ok_or(BankProgramAdoptionPreparationDenial::TargetUnsupported)?;
        let programs = self.request(principal, scope).on_branch(branch).programs();
        let requirements = programs
            .compare(&target)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))?;
        let adoption = programs.adopt(&requirements);
        let inventory = adoption
            .workflow_inventory(maximum_work_units)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))?;
        if expected.branch() != branch
            || inventory.definitions().len() != 1
            || inventory.definitions()[0].entity_id() != expected.entity_id()
        {
            return Err(BankProgramAdoptionPreparationDenial::WorkflowDefinitionMismatch);
        }
        if !inventory.instances().is_empty() {
            return Err(BankProgramAdoptionPreparationDenial::LiveWorkflowInstances);
        }
        let choices = inventory
            .dispositions()
            .definition(
                &inventory.definitions()[0],
                WorthQueryWorkflowDefinitionDisposition::Retire,
            )
            .map_err(BankProgramAdoptionPreparationDenial::WorkflowDisposition)?;
        adoption
            .workflow(choices)
            .prepare(maximum_work_units)
            .map_err(|denial| BankProgramAdoptionPreparationDenial::Query(Box::new(denial)))
    }
}
