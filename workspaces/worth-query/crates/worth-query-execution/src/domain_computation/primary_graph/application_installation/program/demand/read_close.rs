//! Access-sealed cleanup of the actual admitted read interest.
use super::*;
use crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost;
impl<
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        D: WorthQueryApplicationOutputDemand<S>,
    > WorthQueryAdmittedProgramOutput<S, P, D>
{
    pub fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), crate::domain_computation::primary_graph::application_output_demand::denial::read_cleanup::WorthQueryOutputReadCloseDenial>{
        self.admitted
            .close_read_request(phase, runtime.runtime(), branch)
    }
}
