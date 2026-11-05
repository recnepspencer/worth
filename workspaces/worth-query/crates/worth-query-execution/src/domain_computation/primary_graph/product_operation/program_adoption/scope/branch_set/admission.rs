//! Capability preflight over each exact covered source program.
use super::*;
use crate::domain_computation::execution_runtime::ApplicationAdvancementOps;
use crate::domain_computation::primary_graph::product_operation::ContextOps;
pub trait AdmissionOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn authorize_branch_set_adoption(
        &self,
        coverage: &WorthQueryOrderedProgramAdoptionCoverage,
        target: &ApplicationProgramRevision,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), WorthQueryBranchSetAdoptionPreparationDenial> {
        self.with_application_advancement(request, |phase| {
            for &branch in coverage.branches() {
                let selected = self.on_branch(branch).select(&phase).map_err(|denial| {
                    WorthQueryBranchSetAdoptionPreparationDenial::ProductSelection {
                        branch,
                        denial,
                    }
                })?;
                selected
                    .authorize_program_adoption(&phase, principal, request, target)
                    .map_err(
                        |cause| WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
                            branch,
                            denial: WorthQueryBranchAdoptionPreparationDenial::Authorization(cause),
                        },
                    )?;
            }
            Ok(())
        })
        .map_err(WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied)?
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > AdmissionOps<Schema> for HostRuntime
{
}
